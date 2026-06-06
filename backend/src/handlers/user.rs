use actix_web::{patch, post, delete, web, HttpResponse, Responder};
use validator::Validate;
use actix_multipart::Multipart;
use crate::{
    helpers::{api::AuthenticatedUser, images},
    models::{
        app::AppState,
        user::{UpdateUserInfoRequest, User},
    },
};

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
        Ok(None) => HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "user not found" })),
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

    let bucket_name = String::from("avatars");

    // Fetch current avatar from DB to delete it from S3
    let current_avatar: Option<String> = match sqlx::query_scalar::<_, Option<String>>("SELECT avatar_url FROM users WHERE id = $1")
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
            let _ = state.s3_client.delete_object()
                .bucket(&bucket_name)
                .key(filename)
                .send()
                .await;
        }
    }

    // Generate unique key based on user_id and new UUID
    let key = format!("{}_{}.webp", user.id, uuid::Uuid::new_v4());

    // Upload to S3 (MinIO)
    if let Err(e) = state.s3_client.put_object()
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
    let db_result = sqlx::query(
        "UPDATE users SET avatar_url = $1 WHERE id = $2"
    )
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
pub async fn delete_avatar(
    user: AuthenticatedUser,
    state: web::Data<AppState>
) -> impl Responder {
    let bucket_name = String::from("avatars");

    // Fetch current avatar from DB to delete it from S3
    let current_avatar: Option<String> = match sqlx::query_scalar::<_, Option<String>>("SELECT avatar_url FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&state.pool)
        .await 
    {
        Ok(Some(url_opt)) => url_opt,
        _ => None,
    };

    if let Some(old_url) = current_avatar {
        if let Some(filename) = old_url.rsplit('/').next() {
            if let Err(e) = state.s3_client.delete_object()
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
    let db_result = sqlx::query(
        "UPDATE users SET avatar_url = NULL WHERE id = $1"
    )
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