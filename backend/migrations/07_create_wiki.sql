-- ============================================================
-- Migration 07: Wiki system
-- Создаёт три новые сущности:
--   1. wiki_types      — справочник типов wiki-статей (неизменяемый)
--   2. wiki_articles   — основная сущность wiki-статьи
--   3. wiki_stars      — звёзды пользователей для wiki-статей
--
-- Также расширяет функцию update_post_comments_count(), чтобы
-- она поддерживала полиморфную модель комментариев (entity_type)
-- и корректно обновляла comment_count для wiki_articles.
-- ============================================================

BEGIN;

-- ┌─────────────────────────────────────────────────────────┐
-- │  1. wiki_types — справочник типов wiki-статей           │
-- │                                                         │
-- │  Предполагается ограниченный, заранее заполненный       │
-- │  набор значений, который не изменяется в рантайме.      │
-- │  ON DELETE RESTRICT на wiki_articles гарантирует, что   │
-- │  тип нельзя удалить, пока к нему привязаны статьи.      │
-- └─────────────────────────────────────────────────────────┘
CREATE TABLE IF NOT EXISTS wiki_types (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    title      VARCHAR(255) NOT NULL UNIQUE,
    created_at BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT,
    updated_at BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT
);

CREATE TRIGGER update_wiki_types_updated_at
    BEFORE UPDATE ON wiki_types
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();


-- ┌─────────────────────────────────────────────────────────┐
-- │  2. wiki_articles — основная сущность wiki-статьи       │
-- │                                                         │
-- │  created_by     — автор (создатель) статьи. Nullable,   │
-- │                   чтобы при удалении аккаунта статья    │
-- │                   сохранялась (ON DELETE SET NULL).     │
-- │                                                         │
-- │  last_edited_by — UUID последнего редактора. Nullable   │
-- │                   (NULL = статья ещё не редактировалась │
-- │                   после создания, либо редактор удалён).│
-- │                                                         │
-- │  content        — содержимое статьи в формате Markdown. │
-- │                   Тип TEXT — без ограничения по длине.  │
-- │                                                         │
-- │  is_confirmed   — флаг проверки статьи модераторами     │
-- │                   (заготовка, по умолчанию FALSE).      │
-- │                                                         │
-- │  stars_count    — денормализованный счётчик, обновляется│
-- │                   триггером на wiki_stars.              │
-- │                                                         │
-- │  comment_count  — денормализованный счётчик, обновляется│
-- │                   триггером на comments (entity_type =  │
-- │                   'wiki').                              │
-- └─────────────────────────────────────────────────────────┘
CREATE TABLE IF NOT EXISTS wiki_articles (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    title          VARCHAR(255) NOT NULL,
    content        TEXT         NOT NULL,
    wiki_type_id   UUID         NOT NULL REFERENCES wiki_types(id) ON DELETE RESTRICT,
    created_by     UUID         REFERENCES users(id) ON DELETE SET NULL,
    last_edited_by UUID         REFERENCES users(id) ON DELETE SET NULL,
    is_confirmed   BOOLEAN      NOT NULL DEFAULT FALSE,
    stars_count    INTEGER      NOT NULL DEFAULT 0,
    comment_count  INTEGER      NOT NULL DEFAULT 0,
    created_at     BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT,
    updated_at     BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT
);

CREATE INDEX idx_wiki_articles_wiki_type_id ON wiki_articles(wiki_type_id);
CREATE INDEX idx_wiki_articles_created_by   ON wiki_articles(created_by);
CREATE INDEX idx_wiki_articles_created_at   ON wiki_articles(created_at DESC);

CREATE TRIGGER update_wiki_articles_updated_at
    BEFORE UPDATE ON wiki_articles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();


-- ┌─────────────────────────────────────────────────────────┐
-- │  3. wiki_stars — звёзды пользователей для wiki-статей   │
-- │                                                         │
-- │  UNIQUE(wiki_id, user_id) — один пользователь не может  │
-- │  поставить больше одной звезды одной статье.            │
-- │                                                         │
-- │  ON DELETE CASCADE на обоих FK — звезда удаляется вместе│
-- │  со статьёй или аккаунтом пользователя.                 │
-- └─────────────────────────────────────────────────────────┘
CREATE TABLE IF NOT EXISTS wiki_stars (
    id         UUID   PRIMARY KEY DEFAULT gen_random_uuid(),
    wiki_id    UUID   NOT NULL REFERENCES wiki_articles(id) ON DELETE CASCADE,
    user_id    UUID   NOT NULL REFERENCES users(id)         ON DELETE CASCADE,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT,
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::BIGINT,

    CONSTRAINT uq_wiki_star_per_user UNIQUE (wiki_id, user_id)
);

CREATE INDEX idx_wiki_stars_wiki_id ON wiki_stars(wiki_id);
CREATE INDEX idx_wiki_stars_user_id ON wiki_stars(user_id);

CREATE TRIGGER update_wiki_stars_updated_at
    BEFORE UPDATE ON wiki_stars
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();


-- ┌─────────────────────────────────────────────────────────┐
-- │  Триггер: stars_count на wiki_articles                  │
-- │                                                         │
-- │  Автоматически инкрементирует / декрементирует          │
-- │  wiki_articles.stars_count при добавлении или удалении  │
-- │  записи в wiki_stars.                                   │
-- └─────────────────────────────────────────────────────────┘
CREATE OR REPLACE FUNCTION update_wiki_stars_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE wiki_articles SET stars_count = stars_count + 1 WHERE id = NEW.wiki_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE wiki_articles SET stars_count = stars_count - 1 WHERE id = OLD.wiki_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_wiki_stars_count
    AFTER INSERT OR DELETE ON wiki_stars
    FOR EACH ROW
    EXECUTE FUNCTION update_wiki_stars_count();


-- ┌─────────────────────────────────────────────────────────┐
-- │  Расширение триггера comment_count (полиморфная модель) │
-- │                                                         │
-- │  Заменяет функцию update_post_comments_count() из       │
-- │  02_create_posts_and_comments.sql, адаптируя её под     │
-- │  новую схему комментариев (migration 06), где вместо    │
-- │  поля post_id используется пара (entity_type, entity_id)│
-- │                                                         │
-- │  Поддерживаемые entity_type:                            │
-- │    'post'    → posts.comments_count                     │
-- │    'wiki'    → wiki_articles.comment_count              │
-- │    'article' → articles.comment_count (таблица будет    │
-- │                создана в отдельной миграции; ветка       │
-- │                безопасна — PL/pgSQL не проверяет        │
-- │                существование таблиц при компиляции)     │
-- └─────────────────────────────────────────────────────────┘
CREATE OR REPLACE FUNCTION update_post_comments_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.entity_type = 'post' THEN
            UPDATE posts         SET comments_count = comments_count + 1 WHERE id = NEW.entity_id;
        ELSIF NEW.entity_type = 'wiki' THEN
            UPDATE wiki_articles SET comment_count  = comment_count  + 1 WHERE id = NEW.entity_id;
        ELSIF NEW.entity_type = 'article' THEN
            UPDATE articles      SET comment_count  = comment_count  + 1 WHERE id = NEW.entity_id;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.entity_type = 'post' THEN
            UPDATE posts         SET comments_count = comments_count - 1 WHERE id = OLD.entity_id;
        ELSIF OLD.entity_type = 'wiki' THEN
            UPDATE wiki_articles SET comment_count  = comment_count  - 1 WHERE id = OLD.entity_id;
        ELSIF OLD.entity_type = 'article' THEN
            UPDATE articles      SET comment_count  = comment_count  - 1 WHERE id = OLD.entity_id;
        END IF;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Примечание: сам триггер trigger_update_comments_count на таблице comments
-- уже создан в migration 02 и переиспользует эту же функцию.
-- Пересоздавать триггер не нужно — достаточно обновить функцию выше.

COMMIT;
