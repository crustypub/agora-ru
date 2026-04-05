use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub bot_token: String,
    pub jwt_secret: String,
}