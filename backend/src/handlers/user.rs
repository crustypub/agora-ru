use actix_web::{patch, post, delete, web, HttpResponse, Responder};
use validator::Validate;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Cursor;
use image::ImageFormat;

use crate::{
    helpers::api::AuthenticatedUser,
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
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut bytes = Vec::new();
    
    // Read files from multipart request
    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                while let Some(chunk_result) = field.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            bytes.extend_from_slice(&chunk);
                        }
                        Err(e) => {
                            eprintln!("Error reading field chunk: {}", e);
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "error": "Failed to read file chunk"
                            }));
                        }
                    }
                }
                break; // Only process the first field
            }
            Err(e) => {
                eprintln!("Error reading multipart: {}", e);
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to parse multipart request"
                }));
            }
        }
    }

    if bytes.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No file uploaded"
        }));
    }

    // Load image from memory (checks supported photo formats)
    let img = match image::load_from_memory(&bytes) {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid image format. Supported formats: JPEG, PNG, WebP, GIF, BMP"
            }));
        }
    };

    // Resize to 400x400 thumbnail
    let resized = img.thumbnail(400, 400);

    // Convert/encode to WebP format
    let mut webp_bytes = Vec::new();
    if let Err(e) = resized.write_to(&mut Cursor::new(&mut webp_bytes), ImageFormat::WebP) {
        eprintln!("Failed to encode image to WebP: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to process image"
        }));
    }

    let bucket_name = std::env::var("S3_BUCKET_AVATARS").expect("S3_BUCKET_AVATARS must be set");

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
    let public_endpoint = std::env::var("S3_PUBLIC_URL").expect("S3_PUBLIC_URL must be set");
    
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
    let bucket_name = std::env::var("S3_BUCKET_AVATARS").expect("S3_BUCKET_AVATARS must be set");

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