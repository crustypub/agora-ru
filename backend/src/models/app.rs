use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct AppState {
    pub pool: PgPool,
    pub bot_token: String,
    pub jwt_secret: String,
}

pub fn default_page() -> i64 {
    1
}
pub fn default_limit() -> i64 {
    10
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Author {
    pub id: Uuid,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
}
