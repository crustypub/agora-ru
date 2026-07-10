use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TelegramAuthParams {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
    /// Любые другие поля, которые Telegram может добавить в будущем
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AuthSessionRow {
    pub telegram_id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: Option<i64>,
    pub status: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TelegramCheckRequest {
    pub token: String,
}
