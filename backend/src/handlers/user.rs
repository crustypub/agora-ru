use crate::{
    helpers::{api::{AuthenticatedUser, escape_like_pattern}, images},
    models::{
        app::AppState,
        user::{GetUsersParams, UpdateUserInfoRequest, User, ShortUser},
    },
};
use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use validator::Validate;

#[get("/users")]
pub async fn get_users(
    params: web::Query<GetUsersParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();

    let search_pattern = params
        .search_value
        .as_ref()
        .map(|val| format!("%{}%", escape_like_pattern(val.trim())));

    let users_result = sqlx::query_as::<_, ShortUser>(
        r#"
        SELECT
            u.id, u.username, 
            u.first_name, u.last_name, u.avatar_url
        FROM users u
        WHERE ($3::text IS NULL OR 
               u.username ILIKE $3 ESCAPE '\' OR 
               u.first_name ILIKE $3 ESCAPE '\' OR 
               u.last_name ILIKE $3 ESCAPE '\')
        ORDER BY u.username DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .bind(&search_pattern)
    .fetch_all(&state.pool)
    .await;

    let count_result = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) 
        FROM users u
        WHERE ($1::text IS NULL OR 
               u.username ILIKE $1 ESCAPE '\' OR 
               u.first_name ILIKE $1 ESCAPE '\' OR 
               u.last_name ILIKE $1 ESCAPE '\')
        "#,
    )
    .bind(&search_pattern)
    .fetch_one(&state.pool)
    .await;

    match (users_result, count_result) {
        (Ok(users), Ok(total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": users,
                "meta": {
                    "current_page": params.page,
                    "per_page": limit,
                    "total_count": total_count,
                    "total_pages": total_pages,
                    "has_next": params.page < total_pages,
                    "has_previous": params.page > 1
                }
            }))
        }
        (Err(e), _) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch users" }))
        }
        (_, Err(e)) => {
            eprintln!("Count error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch users count" }))
        }
    }
}

#[patch("/user")]
pub async fn update_user_info(
    user: AuthenticatedUser,
    body: web::Json<UpdateUserInfoRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }),
        );
    }

    let result = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            username         = COALESCE($2, username),
            first_name       = COALESCE($3, first_name),
            last_name        = CASE
                WHEN $4 IS NULL THEN last_name
                WHEN $4 = '' THEN NULL
                ELSE $4
            END
        WHERE id = $1
        RETURNING
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            avatar_url,
            created_at,
            updated_at,
            last_login
        "#,
    )
    .bind(author_id)
    .bind(&body.username)
    .bind(&body.first_name)
    .bind(&body.last_name)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(user)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": user,
        })),
        Ok(None) => {
            HttpResponse::Forbidden().json(serde_json::json!({ "error": "user not found" }))
        }
        Err(e) => {
            eprintln!("DB error updating user: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to update user" }))
        }
    }
}

#[post("/user/avatar")]
pub async fn upload_avatar(
    user: AuthenticatedUser,
    payload: Multipart,
    state: web::Data<AppState>,
) -> impl Responder {
    // 300 MB limit
    let max_size = 300 * 1024 * 1024;

    let bytes = match images::read_first_file(payload, max_size).await {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    };

    // Resize to 400x400 thumbnail
    let webp_bytes = match images::resize_and_encode_webp(&bytes, 400, 400) {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    };

    let bucket_name =
        std::env::var("MINIO_BUCKET_AVATARS").expect("MINIO_BUCKET_AVATARS must be set");

    // Fetch current avatar from DB to delete it from S3
    let current_avatar: Option<String> =
        match sqlx::query_scalar::<_, Option<String>>("SELECT avatar_url FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await
        {
            Ok(Some(url_opt)) => url_opt,
            _ => None,
        };

    if let Some(old_url) = current_avatar {
        if let Some(filename) = old_url.rsplit('/').next() {
            // Delete old file from S3 (ignore errors to avoid blocking upload if S3 file was manually removed)
            let _ = state
                .s3_client
                .delete_object()
                .bucket(&bucket_name)
                .key(filename)
                .send()
                .await;
        }
    }

    // Generate unique key based on user_id and new UUID
    let key = format!("{}_{}.webp", user.id, uuid::Uuid::new_v4());

    // Upload to S3 (MinIO)
    if let Err(e) = state
        .s3_client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(webp_bytes.into())
        .content_type("image/webp")
        .send()
        .await
    {
        eprintln!("Failed to upload avatar to S3: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to store avatar in S3"
        }));
    }

    // Determine the public URL of the avatar
    let public_endpoint = match std::env::var("S3_PUBLIC_URL") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("S3_PUBLIC_URL must be set");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "S3 configuration error"
            }));
        }
    };

    let avatar_url = format!("{}/{}/{}", public_endpoint, &bucket_name, &key);

    // Update avatar_url in the database
    let db_result = sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(&avatar_url)
        .bind(user.id)
        .execute(&state.pool)
        .await;

    match db_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "avatar_url": avatar_url
        })),
        Err(e) => {
            eprintln!("Failed to update avatar_url in DB: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update user database profile"
            }))
        }
    }
}

#[delete("/user/avatar")]
pub async fn delete_avatar(user: AuthenticatedUser, state: web::Data<AppState>) -> impl Responder {
    let bucket_name =
        std::env::var("MINIO_BUCKET_AVATARS").expect("MINIO_BUCKET_AVATARS must be set");

    // Fetch current avatar from DB to delete it from S3
    let current_avatar: Option<String> =
        match sqlx::query_scalar::<_, Option<String>>("SELECT avatar_url FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await
        {
            Ok(Some(url_opt)) => url_opt,
            _ => None,
        };

    if let Some(old_url) = current_avatar {
        if let Some(filename) = old_url.rsplit('/').next() {
            if let Err(e) = state
                .s3_client
                .delete_object()
                .bucket(&bucket_name)
                .key(filename)
                .send()
                .await
            {
                eprintln!("Failed to delete avatar from S3: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to delete avatar from S3"
                }));
            }
        }
    }

    // Delete avatar_url in the database
    let db_result = sqlx::query("UPDATE users SET avatar_url = NULL WHERE id = $1")
        .bind(user.id)
        .execute(&state.pool)
        .await;

    match db_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
        })),
        Err(e) => {
            eprintln!("Failed to delete avatar_url in DB: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete avatar_url in DB"
            }))
        }
    }
}
