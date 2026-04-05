use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

pub async fn setup_db() -> sqlx::PgPool {
    // 1. Извлекаем строку подключения, которую Docker собрал для нас
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (it comes from docker-compose environment)");

    // 2. Настраиваем пул соединений
    PgPoolOptions::new()
        .max_connections(5)
        // Даем бэкенду шанс дождаться базы, если healthcheck еще не прошел
        .acquire_timeout(Duration::from_secs(3)) 
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres. Check if the container is running.")
}