use sqlx::PgPool;

use crate::{handlers::auth::TelegramAuthParams, models::user::User};

pub async fn upsert_tg_user(
    pool: &PgPool,
    params: &TelegramAuthParams,
) -> Result<User, sqlx::Error> {
    // Этот запрос сначала пытается вставить строку.
    // Если она есть, он ничего не делает в первой части,
    // но вторая часть (SELECT) гарантирует, что мы вернем данные в любом случае.
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
