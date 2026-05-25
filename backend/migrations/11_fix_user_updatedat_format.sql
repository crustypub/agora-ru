-- 1. Удаляем старый триггер и функцию, чтобы не было конфликтов
DROP TRIGGER IF EXISTS set_timestamp_users ON users;
DROP FUNCTION IF EXISTS trigger_set_timestamp();

-- 2. Пересоздаем/изменяем колонку updated_at на тип BIGINT
-- Если колонка уже создана как TIMESTAMP, мы её дропнем и создадим заново как BIGINT
ALTER TABLE users DROP COLUMN IF EXISTS updated_at;
ALTER TABLE users ADD COLUMN updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM now())::bigint;

-- 3. Создаем новую функцию, которая возвращает UNIX-таймстамп в секундах
CREATE OR REPLACE FUNCTION trigger_set_timestamp_bi()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = EXTRACT(EPOCH FROM now())::bigint;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 4. Вешаем обновленный триггер на таблицу users
CREATE TRIGGER set_timestamp_users
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp_bi();