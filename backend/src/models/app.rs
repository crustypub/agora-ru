use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Направление сортировки — переиспользуется во всех эндпоинтах
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    /// Возвращает SQL-ключевое слово для ORDER BY
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

/// Трейт для любого enum, который представляет поле сортировки.
/// Реализуй его в каждом модуле для своего типа.
pub trait SortField {
    /// Возвращает безопасное SQL-имя колонки (только белый список — защита от injection)
    fn as_sql_column(&self) -> &'static str;
}

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
