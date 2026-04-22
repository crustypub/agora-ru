use serde::{Deserialize, Serialize};
use sqlx::{types::{chrono, Json}, FromRow};
use uuid::Uuid;
use validator::Validate;

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    10
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub id: Uuid,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Post {
    pub id: Uuid,
    pub author: Json<Author>,
    pub title: String,
    pub content: String,
    pub rating_plus: i32,
    pub rating_minus: i32,
    pub comments_count: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CreatePost {
    pub id: Uuid,
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub rating_plus: i32,
    pub rating_minus: i32,
    pub comments_count: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
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
