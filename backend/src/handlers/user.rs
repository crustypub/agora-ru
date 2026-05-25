use actix_web::{patch, web, HttpResponse, Responder};
use validator::Validate;

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
            last_name        = COALESCE($4, last_name)
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
