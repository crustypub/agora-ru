use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow, Serialize)]
pub struct WikiType {
    pub id: i32,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}


#[derive(Debug, FromRow, Deserialize, Serialize, Validate)]
pub struct WikiTypeResponse {
    pub id: i32,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWikiArticleRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(length(min = 1, max = 100000))]
    pub content: String,

    pub wiki_type_id: i32,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CreateWikiArticle {
    pub id: Uuid,
    pub title: String,
    pub wiki_type_id: i32,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CreateWikiArticleResponse {
    pub id: Uuid,
    pub title: String,
    pub wiki_type: WikiTypeResponse,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Wiki {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub wiki_type_id: i32,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct WikiResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub wiki_type: WikiTypeResponse,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
