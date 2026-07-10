use sqlx::PgPool;
use uuid::Uuid;
use crate::models::post::{CreatePost, Post};

pub async fn get_posts_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Post>, i64), sqlx::Error> {
    let posts = sqlx::query_as::<_, Post>(
        r#"
        SELECT
            p.id, p.title, p.content,
            p.rating_plus, p.rating_minus, p.comments_count,
            p.created_at, p.updated_at,
            json_build_object(
                'id', u.id,
                'username', u.username,
                'first_name', u.first_name,
                'last_name', u.last_name,
                'avatar_url', u.avatar_url
            ) as author
        FROM posts p
        JOIN users u ON p.author = u.id
        ORDER BY p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await?;

    Ok((posts, count))
}

pub async fn create_post(
    pool: &PgPool,
    author_id: Uuid,
    title: &str,
    content: &str,
) -> Result<CreatePost, sqlx::Error> {
    sqlx::query_as::<_, CreatePost>(
        r#"
        INSERT INTO posts (author, title, content)
        VALUES ($1, $2, $3)
        RETURNING
            id,
            author,
            title,
            content,
            rating_plus,
            rating_minus,
            comments_count,
            created_at,
            updated_at
        "#,
    )
    .bind(author_id)
    .bind(title)
    .bind(content)
    .fetch_one(pool)
    .await
}

pub async fn add_post_rating_like(
    pool: &PgPool,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE posts
        SET
            rating_plus = array_append(array_remove(rating_plus, $1), $1),
            rating_minus = array_remove(rating_minus, $1)
        WHERE id = $2
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn add_post_rating_dislike(
    pool: &PgPool,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE posts
        SET
            rating_minus = array_append(array_remove(rating_minus, $1), $1),
            rating_plus = array_remove(rating_plus, $1)
        WHERE id = $2
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn remove_post_rating_like(
    pool: &PgPool,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE posts
        SET
            rating_plus = array_remove(rating_plus, $1)
        WHERE id = $2
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn remove_post_rating_dislike(
    pool: &PgPool,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE posts
        SET
            rating_minus = array_remove(rating_minus, $1)
        WHERE id = $2
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
