-- Создание расширений (если нужны)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Создание таблиц для Telegram Auth (если модуль требует)
-- Обычно nuxt-telegram-auth сам создаёт таблицы, но можно добавить свои

-- Пример: таблица пользователей форума
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
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