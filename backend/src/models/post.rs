use crate::models::app::{default_limit, default_page, Author};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize)]
pub enum PostRatingMode {
    Increment,
    Decrement,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PostRatingOperationType {
    Add,
    Remove,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Post {
    pub id: Uuid,
    pub author: Json<Author>,
    pub title: String,
    pub content: String,
    pub rating_plus: Vec<Uuid>,
    pub rating_minus: Vec<Uuid>,
    pub comments_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub author: Json<Author>,
    pub title: String,
    pub content: String,
    pub rating_plus: usize,
    pub rating_minus: usize,
    pub comments_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_liked: bool,
    pub is_disliked: bool,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CreatePost {
    pub id: Uuid,
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub rating_plus: Vec<Uuid>,
    pub rating_minus: Vec<Uuid>,
    pub comments_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Debug, FromRow, Serialize)]
pub struct CreatePostResponse {
    pub id: Uuid,
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct PostParams {
    pub author_id: Option<String>,
    pub search_value: Option<String>,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_limit")]
    pub limit: i64,
}

impl PostParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PostRatingRequest {
    pub post_id: Uuid,
    pub mode: PostRatingMode,
    pub operation_type: PostRatingOperationType,
}
