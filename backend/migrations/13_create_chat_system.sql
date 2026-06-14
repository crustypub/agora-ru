-- 1. Создание перечислений (типы чатов и роли участников)
CREATE TYPE room_type AS ENUM ('direct', 'group');
CREATE TYPE room_role AS ENUM ('owner', 'moderator', 'member');

-- 2. Таблица комнат (чатов)
CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type room_type NOT NULL DEFAULT 'direct',
    name VARCHAR(255),               -- NULL для личных чатов
    description TEXT,                -- NULL для личных чатов
    
    -- Уникальный ключ для контроля личных переписок (предотвращает дубликаты)
    direct_key VARCHAR(73) UNIQUE CHECK (
        (type = 'direct' AND direct_key IS NOT NULL) OR 
        (type = 'group' AND direct_key IS NULL)
    ),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 3. Таблица участников чата
CREATE TABLE IF NOT EXISTS room_members (
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role room_role NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_read_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (room_id, user_id)
);

-- Индекс для быстрого поиска комнат, в которых состоит пользователь
CREATE INDEX idx_room_members_user ON room_members(user_id);

-- 4. Таблица сообщений
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL, -- SET NULL сохраняет сообщение при удалении профиля
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE -- Для удаления сообщений "у всех" (Soft Delete)
);

-- Индекс для пагинации и сортировки сообщений в комнатах
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC);

-- 5. Таблица скрытых сообщений (удаленных лично "у себя")
CREATE TABLE IF NOT EXISTS deleted_messages (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (message_id, user_id)
);