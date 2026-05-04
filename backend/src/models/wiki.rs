use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow, Serialize)]
pub struct WikiType {
    pub id: Uuid,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}


#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct WikiTypeResponse {
    pub id: Uuid,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}
