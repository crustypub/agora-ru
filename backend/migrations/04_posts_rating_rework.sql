BEGIN;

-- 1. Удаляем старые колонки (так как данные нам не нужны)
ALTER TABLE posts DROP COLUMN IF EXISTS rating_plus;
ALTER TABLE posts DROP COLUMN IF EXISTS rating_minus;

-- 2. Добавляем их заново с нужным типом
ALTER TABLE posts ADD COLUMN rating_plus UUID[] DEFAULT '{}';
ALTER TABLE posts ADD COLUMN rating_minus UUID[] DEFAULT '{}';

COMMIT;