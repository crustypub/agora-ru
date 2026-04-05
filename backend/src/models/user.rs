use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub last_login: Option<chrono::NaiveDateTime>,
}
