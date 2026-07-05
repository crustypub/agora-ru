CREATE TABLE IF NOT EXISTS message_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_key VARCHAR(512) NOT NULL,    
    file_name VARCHAR(255) NOT NULL,   
    file_size BIGINT NOT NULL,         
    file_mime VARCHAR(128) NOT NULL,   
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Индекс для быстрой сборки сообщений при пагинации
CREATE INDEX idx_message_attachments_msg ON message_attachments(message_id);