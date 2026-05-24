-- ============================================================
-- Migration 09: wiki_types.id UUID → SERIAL (integer)
--
-- Причина: переход от UUID к последовательному целочисленному
-- первичному ключу для упрощения работы со справочником.
--
-- Порядок действий:
--   1. Очищаем wiki_articles (FK на wiki_types → ON DELETE RESTRICT)
--   2. Очищаем wiki_types
--   3. Пересоздаём wiki_types с id SERIAL
--   4. Обновляем тип wiki_type_id в wiki_articles
--   5. Повторно наполняем wiki_types теми же значениями,
--      что и миграция 08, теперь с явными числовыми id.
-- ============================================================

BEGIN;

-- ┌─────────────────────────────────────────────────────────┐
-- │  1. Очистка зависимых данных                            │
-- │                                                         │
-- │  wiki_articles ссылается на wiki_types(id) с            │
-- │  ON DELETE RESTRICT, поэтому сначала удаляем статьи.   │
-- │  wiki_stars удалятся каскадно (ON DELETE CASCADE).      │
-- └─────────────────────────────────────────────────────────┘
DELETE FROM wiki_articles;

-- ┌─────────────────────────────────────────────────────────┐
-- │  2. Очистка справочника                                 │
-- └─────────────────────────────────────────────────────────┘
DELETE FROM wiki_types;

-- ┌─────────────────────────────────────────────────────────┐
-- │  3. Замена первичного ключа wiki_types: UUID → SERIAL   │
-- └─────────────────────────────────────────────────────────┘

-- 3a. Снимаем FK-ограничение с wiki_articles
ALTER TABLE wiki_articles
    DROP CONSTRAINT wiki_articles_wiki_type_id_fkey;

-- 3b. Меняем тип id в wiki_types
ALTER TABLE wiki_types
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE INTEGER USING id::TEXT::INTEGER;

-- (SERIAL = INTEGER + последовательность)
CREATE SEQUENCE IF NOT EXISTS wiki_types_id_seq OWNED BY wiki_types.id;
ALTER TABLE wiki_types
    ALTER COLUMN id SET DEFAULT nextval('wiki_types_id_seq');

-- ┌─────────────────────────────────────────────────────────┐
-- │  4. Обновление wiki_articles.wiki_type_id: UUID → INT  │
-- └─────────────────────────────────────────────────────────┘
ALTER TABLE wiki_articles
    ALTER COLUMN wiki_type_id TYPE INTEGER USING wiki_type_id::TEXT::INTEGER;

-- Восстанавливаем FK
ALTER TABLE wiki_articles
    ADD CONSTRAINT wiki_articles_wiki_type_id_fkey
        FOREIGN KEY (wiki_type_id) REFERENCES wiki_types(id) ON DELETE RESTRICT;

-- ┌─────────────────────────────────────────────────────────┐
-- │  5. Повторное наполнение справочника (явные id)          │
-- └─────────────────────────────────────────────────────────┘
INSERT INTO wiki_types (id, title) VALUES
    (1, 'Легализация и документы'),
    (2, 'Жильё и быт'),
    (3, 'Медицина и страхование'),
    (4, 'Работа и налоги'),
    (5, 'Безопасность и чрезвычайные ситуации'),
    (6, 'Другое')
ON CONFLICT (title) DO NOTHING;

-- Синхронизируем последовательность после явной вставки
SELECT setval('wiki_types_id_seq', (SELECT MAX(id) FROM wiki_types));

COMMIT;
