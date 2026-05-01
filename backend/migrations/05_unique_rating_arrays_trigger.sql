BEGIN;

-- Функция для удаления дубликатов из массива UUID
CREATE OR REPLACE FUNCTION array_unique_uuid(arr UUID[])
RETURNS UUID[] AS $$
BEGIN
    IF arr IS NULL THEN
        RETURN NULL;
    END IF;
    
    RETURN (
        SELECT ARRAY(
            SELECT DISTINCT unnest(arr)
        )
    );
END;
$$ LANGUAGE plpgsql;

-- Триггерная функция для очистки массивов от дубликатов
CREATE OR REPLACE FUNCTION enforce_unique_rating_arrays()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.rating_plus IS NOT NULL THEN
        NEW.rating_plus := array_unique_uuid(NEW.rating_plus);
    END IF;

    IF NEW.rating_minus IS NOT NULL THEN
        NEW.rating_minus := array_unique_uuid(NEW.rating_minus);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Создаем сам триггер для таблицы posts
DROP TRIGGER IF EXISTS trg_enforce_unique_rating_arrays ON posts;

CREATE TRIGGER trg_enforce_unique_rating_arrays
BEFORE INSERT OR UPDATE OF rating_plus, rating_minus ON posts
FOR EACH ROW
EXECUTE FUNCTION enforce_unique_rating_arrays();

COMMIT;
