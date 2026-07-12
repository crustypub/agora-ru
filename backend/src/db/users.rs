use sqlx::PgPool;
use uuid::Uuid;
use crate::models::auth::TelegramAuthParams;
use crate::models::user::{ShortUser, User};

pub async fn upsert_tg_user(
    pool: &PgPool,
    params: &TelegramAuthParams,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        WITH inserted AS (
            INSERT INTO users (telegram_id, first_name, username)
            VALUES ($1, $2, $3)
            ON CONFLICT (telegram_id) DO NOTHING
            RETURNING *
        )
        SELECT * FROM inserted
        UNION ALL
        SELECT * FROM users 
        WHERE telegram_id = $1
        LIMIT 1
        "#,
    )
    .bind(params.id)
    .bind(&params.first_name)
    .bind(&params.username)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_users_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    search_pattern: Option<&str>,
) -> Result<(Vec<ShortUser>, i64), sqlx::Error> {
    let users = sqlx::query_as::<_, ShortUser>(
        r#"
        SELECT id, username, first_name, last_name, avatar_url, description
        FROM users
        WHERE ($3::text IS NULL OR 
               username ILIKE $3 ESCAPE '\' OR 
               first_name ILIKE $3 ESCAPE '\' OR 
               last_name ILIKE $3 ESCAPE '\')
        ORDER BY username DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .bind(search_pattern)
    .fetch_all(pool)
    .await?;

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) 
        FROM users
        WHERE ($1::text IS NULL OR 
               username ILIKE $1 ESCAPE '\' OR 
               first_name ILIKE $1 ESCAPE '\' OR 
               last_name ILIKE $1 ESCAPE '\')
        "#,
    )
    .bind(search_pattern)
    .fetch_one(pool)
    .await?;

    Ok((users, count))
}

pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    username: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            username         = COALESCE($2, username),
            first_name       = COALESCE($3, first_name),
            last_name        = CASE
                WHEN $4 IS NULL THEN last_name
                WHEN $4 = '' THEN NULL
                ELSE $4
            END
        WHERE id = $1
        RETURNING
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            avatar_url,
            created_at,
            last_login
        "#,
    )
    .bind(user_id)
    .bind(username)
    .bind(first_name)
    .bind(last_name)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_avatar_url(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let res: Option<Option<String>> = sqlx::query_scalar(
        "SELECT avatar_url FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(res.flatten())
}

pub async fn update_user_avatar_url(
    pool: &PgPool,
    user_id: Uuid,
    avatar_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(avatar_url)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
