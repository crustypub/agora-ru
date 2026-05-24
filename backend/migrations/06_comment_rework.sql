-- 1. Добавляем новые полиморфные поля
ALTER TABLE comments 
ADD COLUMN entity_type VARCHAR(50),
ADD COLUMN entity_id UUID;

-- 2. Заполняем их данными из старых постов
UPDATE comments 
SET 
    entity_type = 'post',
    entity_id = post_id
WHERE post_id IS NOT NULL;

-- 3. Делаем новые поля NOT NULL (после заполнения)
ALTER TABLE comments 
ALTER COLUMN entity_type SET NOT NULL,
ALTER COLUMN entity_id SET NOT NULL;

-- 4. Удаляем старый внешний ключ и поле
ALTER TABLE comments DROP CONSTRAINT comments_post_id_fkey;
ALTER TABLE comments DROP COLUMN post_id;

-- 5. Создаём индексы для быстрого поиска
CREATE INDEX idx_comments_entity ON comments(entity_type, entity_id);
CREATE INDEX idx_comments_entity_id ON comments(entity_id);

-- 6. (Опционально) Добавляем CHECK constraint для валидации типов
ALTER TABLE comments 
ADD CONSTRAINT chk_entity_type 
CHECK (entity_type IN ('post', 'article', 'wiki'));