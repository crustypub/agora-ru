use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author: Uuid,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}
