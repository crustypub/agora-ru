use crate::models::app::{default_limit, default_page, Author, SortField, SortOrder};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
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

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWikiArticleRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    #[validate(length(min = 1, max = 100000))]
    pub content: Option<String>,

    pub wiki_type_id: Option<i32>,
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Wiki {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub wiki_type: Json<WikiTypeResponse>,
    pub created_by: Json<Author>,
    pub last_edited_by: Json<Author>,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub stars_count: i32,
    pub comment_count: i32,
    pub is_starred: bool,
}
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct WikiListItem {
    pub id: Uuid,
    pub title: String,
    pub wiki_type: Json<WikiTypeResponse>,
    pub created_by: Json<Author>,
    pub last_edited_by: Json<Author>,
    pub is_confirmed: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub stars_count: i32,
    pub comment_count: i32,
    pub is_starred: bool,
}


/// Поля сортировки для wiki-статей
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum WikiSortField {
    CreatedAt,
    UpdatedAt,
    StarsCount,
}

impl SortField for WikiSortField {
    fn as_sql_column(&self) -> &'static str {
        match self {
            WikiSortField::CreatedAt => "wa.created_at",
            WikiSortField::UpdatedAt => "wa.updated_at",
            WikiSortField::StarsCount => "wa.stars_count",
        }
    }
}

impl Default for WikiSortField {
    fn default() -> Self {
        WikiSortField::CreatedAt
    }
}

#[derive(Deserialize)]
pub struct WikIArticlesParams {
    pub author_id: Option<String>,

    /// Фильтр по типу вики
    pub wiki_type: Option<i32>,

    /// Фильтр по статусу подтверждения
    pub is_confirmed: Option<bool>,

    /// Полнотекстовый поиск по title и content
    pub search: Option<String>,

    /// Поле сортировки
    #[serde(default)]
    pub sort_by: WikiSortField,

    /// Направление сортировки
    #[serde(default)]
    pub sort_order: SortOrder,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_limit")]
    pub limit: i64,
}
impl WikIArticlesParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct CreateWikiStar {
    pub id: Uuid,
    pub wiki_id: Uuid,
    pub user_id: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
}