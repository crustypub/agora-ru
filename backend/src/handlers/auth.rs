use std::collections::HashMap;

use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db::users::upsert_tg_user;
use crate::helpers::telegram::verify_tg_hash;
use crate::models::app::AppState;

#[derive(Deserialize, Serialize)]
pub struct TelegramAuthParams {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
    /// Любые другие поля, которые Telegram может добавить в будущем
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[post("/auth/telegram")]
pub async fn telegram_auth(
    params: web::Json<TelegramAuthParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 1. Проверяем подпись
    if !verify_tg_hash(&params, &state.bot_token) {
        return HttpResponse::Unauthorized().finish();
    }

    // 2. Ищем или создаем пользователя
    let user = match upsert_tg_user(&state.pool, &params).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to upsert user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 3. Выпускаем JWT
    let secret = &state.jwt_secret;

    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + 60 * 60 * 24 * 7; // 7 days

    let claims = Claims {
        sub: user.id,
        exp: expiration as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create token: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(AuthResponse { token })
}
