use crate::models::app::{default_limit, default_page, Author};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub author: Uuid,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub author: Json<Author>,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 50))]
    pub entity_type: String,

    pub entity_id: Uuid,

    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

#[derive(Deserialize)]
pub struct CommentParams {
    pub entity_type: String,
    pub entity_id: Uuid,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_limit")]
    pub limit: i64,
}

impl CommentParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}
