use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use sqlx::{FromRow};

#[derive(Debug, FromRow, Serialize, Deserialize)]

pub struct User {
    pub id: Uuid,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserInfoRequest {
    #[validate(length(min = 3, max = 16))]
    pub username: Option<String>,

    #[validate(length(min = 1, max = 32))]
    pub first_name: Option<String>,

    #[validate(length(max = 32))]
    pub last_name: Option<String>,
}