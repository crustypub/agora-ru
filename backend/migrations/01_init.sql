-- Создание расширений (если нужны)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Пример: таблица пользователей форума
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    telegram_id BIGINT UNIQUE NOT NULL,
    username VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    photo_url TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

-- Индексы для производительности
CREATE INDEX idx_telegram_id ON users(telegram_id);