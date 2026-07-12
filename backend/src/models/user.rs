use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use sqlx::{FromRow};
use crate::models::app::{default_limit, default_page};


#[derive(Deserialize)]
pub struct GetUsersParams {
    pub search_value: Option<String>,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_limit")]
    pub limit: i64,
}

impl GetUsersParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}


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
    pub description: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ShortUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
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