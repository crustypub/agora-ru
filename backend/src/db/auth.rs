use sqlx::PgPool;
use uuid::Uuid;
use crate::models::app::Author;
use crate::models::auth::AuthSessionRow;

pub async fn get_author_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<Author>, sqlx::Error> {
    sqlx::query_as::<_, Author>(
        "SELECT id, username, first_name, last_name, avatar_url FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_pending_token(
    pool: &PgPool,
    token: &str,
    created_at: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO auth_tokens (token, status, created_at) VALUES ($1, 'pending', $2)"
    )
    .bind(token)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn clean_expired_tokens(
    pool: &PgPool,
    expire_limit: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM auth_tokens WHERE created_at < $1")
        .bind(expire_limit)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_auth_session(
    pool: &PgPool,
    token: &str,
) -> Result<Option<AuthSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, AuthSessionRow>(
        "SELECT telegram_id, first_name, last_name, username, photo_url, auth_date, status 
         FROM auth_tokens 
         WHERE token = $1"
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn delete_auth_token(
    pool: &PgPool,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM auth_tokens WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;

    Ok(())
}
