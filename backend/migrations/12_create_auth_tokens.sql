CREATE TABLE IF NOT EXISTS auth_tokens (
    token VARCHAR(64) PRIMARY KEY,
    telegram_id BIGINT,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    username VARCHAR(255),
    photo_url TEXT,
    auth_date BIGINT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM now())::bigint
);
