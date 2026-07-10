use sqlx::PgPool;
use uuid::Uuid;
use crate::models::comment::{Comment, CommentResponse};

pub async fn get_comments_paginated(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<CommentResponse>, i64), sqlx::Error> {
    let comments = sqlx::query_as::<_, CommentResponse>(
        r#"
        SELECT
            c.id,
            c.entity_type,
            c.entity_id,
            c.content,
            c.created_at,
            c.updated_at,
            json_build_object(
                'id',         u.id,
                'username',   u.username,
                'first_name', u.first_name,
                'last_name',  u.last_name,
                'avatar_url', u.avatar_url
            ) AS author
        FROM comments c
        JOIN users u ON c.author = u.id
        WHERE c.entity_type = $1 AND c.entity_id = $2
        ORDER BY c.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM comments WHERE entity_type = $1 AND entity_id = $2",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_one(pool)
    .await?;

    Ok((comments, count))
}

pub async fn create_comment(
    pool: &PgPool,
    author_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    content: &str,
) -> Result<Comment, sqlx::Error> {
    sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (entity_type, entity_id, author, content)
        VALUES ($1, $2, $3, $4)
        RETURNING id, entity_type, entity_id, author, content, created_at, updated_at
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(author_id)
    .bind(content)
    .fetch_one(pool)
    .await
}

pub async fn get_comment_author(
    pool: &PgPool,
    comment_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT author FROM comments WHERE id = $1"
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await
}

pub async fn update_comment(
    pool: &PgPool,
    comment_id: Uuid,
    content: &str,
) -> Result<Comment, sqlx::Error> {
    sqlx::query_as::<_, Comment>(
        r#"
        UPDATE comments
        SET content = $1
        WHERE id = $2
        RETURNING id, entity_type, entity_id, author, content, created_at, updated_at
        "#,
    )
    .bind(content)
    .bind(comment_id)
    .fetch_one(pool)
    .await
}

pub async fn delete_comment(
    pool: &PgPool,
    comment_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(comment_id)
        .execute(pool)
        .await?;

    Ok(())
}
