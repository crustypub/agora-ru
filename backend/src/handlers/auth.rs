use std::collections::HashMap;

use actix_web::{
    get, post, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

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

#[post("/auth/telegram")]
pub async fn telegram_auth(
    params: web::Json<TelegramAuthParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 1. Проверяем подпись (бизнес-логика в слое Service)

    if !verify_tg_hash(&params, &state.bot_token) {
        return format!("not valid");
    }
    return format!("valid");

    // // 2. Ищем или создаем пользователя
    // let user = user_repository::upsert_tg_user(&pool, &params).await;

    // // 3. Выпускаем JWT
    // let token = jwt_service::create_token(user.id);

    // HttpResponse::Ok().json(AuthResponse { token })
}
