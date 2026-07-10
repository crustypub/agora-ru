use crate::models::chat::{
    AttachmentResponse, ChatListItem, ChatMessage, ReadRoomPayload, RoomMemberInfo, SendMessagePayload, WsMessage,
};
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use actix_ws::Message;
use futures_util::StreamExt;
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Кастомное перечисление ошибок для чат-системы.
/// Реализует `ResponseError`, что позволяет автоматически преобразовывать ошибки
/// бизнес-логики и базы данных в соответствующие HTTP-ответы.
#[derive(Debug)]
pub enum ChatError {
    Database(sqlx::Error),
    Validation(String),
    NotFound(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatError::Database(e) => write!(f, "Database error: {}", e),
            ChatError::Validation(e) => write!(f, "Validation failed: {}", e),
            ChatError::NotFound(e) => write!(f, "Not found: {}", e),
            ChatError::Forbidden(e) => write!(f, "Forbidden: {}", e),
            ChatError::BadRequest(e) => write!(f, "Bad request: {}", e),
            ChatError::Conflict(e) => write!(f, "Conflict: {}", e),
        }
    }
}

impl ResponseError for ChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            ChatError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatError::Validation(_) => StatusCode::BAD_REQUEST,
            ChatError::NotFound(_) => StatusCode::NOT_FOUND,
            ChatError::Forbidden(_) => StatusCode::FORBIDDEN,
            ChatError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ChatError::Conflict(_) => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = match self {
            ChatError::Database(e) => {
                eprintln!("Database error occurred in chats helper: {:?}", e);
                "Internal server error".to_string()
            }
            other => other.to_string(),
        };

        HttpResponse::build(status).json(serde_json::json!({
            "status": "error",
            "error": message
        }))
    }
}

impl From<sqlx::Error> for ChatError {
    fn from(err: sqlx::Error) -> Self {
        ChatError::Database(err)
    }
}

pub struct PaginatedRooms {
    pub rooms: Vec<ChatListItem>,
    pub total_count: i64,
}

pub async fn get_user_rooms_paginated(
    db: &sqlx::PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
    search_pattern: Option<String>,
) -> Result<PaginatedRooms, ChatError> {
    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        WHERE rm.user_id = $1
          AND (
              $2::text IS NULL OR (
                  r.name ILIKE $2 OR
                  (
                      r.type = 'direct' AND EXISTS (
                          SELECT 1 FROM room_members orm
                          JOIN users ou ON orm.user_id = ou.id
                          WHERE orm.room_id = r.id AND orm.user_id != $1
                            AND (
                                ou.username ILIKE $2 OR
                                ou.first_name ILIKE $2 OR
                                ou.last_name ILIKE $2
                            )
                      )
                  )
              )
          )
        "#,
        user_id,
        search_pattern
    )
    .fetch_one(db)
    .await?;

    let rooms = sqlx::query_as::<_, ChatListItem>(
        r#"
        SELECT
            r.id,
            r.type::text as room_type,
            r.name,
            r.description,
            r.direct_key,
            EXTRACT(EPOCH FROM r.created_at)::BIGINT as created_at,
            EXTRACT(EPOCH FROM r.updated_at)::BIGINT as updated_at,
            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.room_id = r.id
                  AND m.created_at > rm.last_read_at
                  AND (m.sender_id IS NULL OR m.sender_id != $1)
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
            ) AS unread_count,
            (
                SELECT json_build_object(
                    'id', m.id,
                    'room_id', m.room_id,
                    'sender_id', m.sender_id,
                    'content', m.content,
                    'created_at', EXTRACT(EPOCH FROM m.created_at)::BIGINT,
                    'author', json_build_object(
                        'id', u.id,
                        'username', u.username,
                        'first_name', u.first_name,
                        'last_name', u.last_name,
                        'avatar_url', u.avatar_url
                    ),
                    'is_read', CASE
                        WHEN m.sender_id = $1 THEN 
                            EXISTS (
                                SELECT 1 
                                FROM room_members rm_other 
                                WHERE rm_other.room_id = m.room_id 
                                  AND rm_other.user_id != $1 
                                  AND rm_other.last_read_at >= m.created_at
                            )
                        ELSE 
                            EXISTS (
                                SELECT 1 
                                FROM room_members rm_self 
                                WHERE rm_self.room_id = m.room_id 
                                  AND rm_self.user_id = $1 
                                  AND rm_self.last_read_at >= m.created_at
                            )
                    END
                )
                FROM messages m
                LEFT JOIN users u ON m.sender_id = u.id
                WHERE m.room_id = r.id
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
                ORDER BY m.created_at DESC
                LIMIT 1
            ) AS last_message,
            CASE 
                WHEN r.type = 'direct' THEN (
                    SELECT json_build_object(
                        'id', ou.id,
                        'username', ou.username,
                        'first_name', ou.first_name,
                        'last_name', ou.last_name,
                        'avatar_url', ou.avatar_url
                    )
                    FROM room_members orm
                    JOIN users ou ON orm.user_id = ou.id
                    WHERE orm.room_id = r.id AND orm.user_id != $1
                    LIMIT 1
                )
                ELSE NULL
            END AS direct_user
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        WHERE rm.user_id = $1
          AND (
              $4::text IS NULL OR (
                  r.name ILIKE $4 OR
                  (
                      r.type = 'direct' AND EXISTS (
                          SELECT 1 FROM room_members orm
                          JOIN users ou ON orm.user_id = ou.id
                          WHERE orm.room_id = r.id AND orm.user_id != $1
                            AND (
                                ou.username ILIKE $4 OR
                                ou.first_name ILIKE $4 OR
                                ou.last_name ILIKE $4
                            )
                      )
                  )
              )
          )
        ORDER BY COALESCE(
            (
                SELECT MAX(m.created_at)
                FROM messages m
                WHERE m.room_id = r.id
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
            ),
            r.updated_at
        ) DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .bind(search_pattern)
    .fetch_all(db)
    .await?;

    Ok(PaginatedRooms { rooms, total_count })
}

pub async fn create_direct_room(
    db: &sqlx::PgPool,
    author_id: Uuid,
    user_2: Uuid,
) -> Result<(Uuid, bool), ChatError> {
    if author_id == user_2 {
        return Err(ChatError::BadRequest(
            "Cannot create direct chat with yourself".to_string(),
        ));
    }

    // Алфавитный ключ для уникальности переписки двух пользователей
    let (min_user, max_user) = if author_id < user_2 {
        (author_id, user_2)
    } else {
        (user_2, author_id)
    };
    let direct_key = format!("{}:{}", min_user, max_user);

    // Проверяем, существует ли уже такой чат
    let existing_room =
        sqlx::query_scalar!("SELECT id FROM rooms WHERE direct_key = $1", direct_key)
            .fetch_optional(db)
            .await?;

    if let Some(room_id) = existing_room {
        return Ok((room_id, true));
    }

    let mut tx = db.begin().await?;

    // Создаем комнату с уникальным ключом
    let room_id = sqlx::query_scalar!(
        r#"
        INSERT INTO rooms (type, direct_key)
        VALUES ($1::room_type, $2)
        ON CONFLICT (direct_key) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
        "direct" as &str,
        direct_key
    )
    .fetch_one(&mut *tx)
    .await?;

    // Добавляем создателя в участники
    sqlx::query!(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
        room_id,
        author_id,
        "member" as &str
    )
    .execute(&mut *tx)
    .await?;

    // Добавляем собеседника в участники
    sqlx::query!(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
        room_id,
        user_2,
        "member" as &str
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((room_id, false))
}

/// Создает групповую комнату и делает создателя ее владельцем (owner).
pub async fn create_group_room(
    db: &sqlx::PgPool,
    author_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Uuid, ChatError> {
    let mut tx = db.begin().await?;

    // Создаем групповую комнату
    let room_id = sqlx::query_scalar!(
        r#"
        INSERT INTO rooms (type, name, description)
        VALUES ($1::room_type, $2, $3)
        RETURNING id
        "#,
        "group" as &str,
        name,
        description
    )
    .fetch_one(&mut *tx)
    .await?;

    // Назначаем создателя владельцем
    sqlx::query!(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        "#,
        room_id,
        author_id,
        "owner" as &str
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(room_id)
}

pub async fn add_room_member(
    db: &sqlx::PgPool,
    requester_id: Uuid,
    room_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), ChatError> {
    let room_info = sqlx::query!(
        r#"
        SELECT type::text as room_type,
               (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role
        FROM rooms WHERE id = $1
        "#,
        room_id,
        requester_id
    )
    .fetch_optional(db)
    .await?;

    let info = room_info.ok_or_else(|| ChatError::NotFound("Room not found".to_string()))?;

    if info.room_type.as_deref() == Some("direct") {
        return Err(ChatError::BadRequest(
            "Cannot add members to direct chats".to_string(),
        ));
    }

    let role = info.requester_role.ok_or_else(|| {
        ChatError::Forbidden("You are not a member of this chat room".to_string())
    })?;

    // Приглашать могут только владельцы и модераторы
    if role != "owner" && role != "moderator" {
        return Err(ChatError::Forbidden(
            "You do not have permission to invite members (must be owner or moderator)".to_string(),
        ));
    }

    let user_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as \"exists!\"",
        target_user_id
    )
    .fetch_one(db)
    .await
    .unwrap_or(false);

    if !user_exists {
        return Err(ChatError::NotFound("User to invite not found".to_string()));
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
        room_id,
        target_user_id,
        "member" as &str
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        Err(ChatError::Conflict(
            "User is already a member of this room".to_string(),
        ))
    } else {
        Ok(())
    }
}

pub async fn remove_room_member(
    db: &sqlx::PgPool,
    requester_id: Uuid,
    room_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), ChatError> {
    // Получаем роли и тип комнаты одним запросом
    let roles = sqlx::query!(
        r#"
        SELECT 
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role,
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $3) as target_role,
            (SELECT type::text FROM rooms WHERE id = $1) as room_type
        "#,
        room_id,
        requester_id,
        target_user_id
    )
    .fetch_optional(db)
    .await?;

    let roles_info = roles
        .ok_or_else(|| ChatError::NotFound("Room or member information not found".to_string()))?;

    if roles_info.room_type.as_deref() == Some("direct") {
        return Err(ChatError::BadRequest(
            "Cannot modify members in direct chats".to_string(),
        ));
    }

    let req_role = roles_info.requester_role.ok_or_else(|| {
        ChatError::Forbidden("You are not a member of this chat room".to_string())
    })?;

    let tgt_role = roles_info.target_role.ok_or_else(|| {
        ChatError::NotFound("Target user is not a member of this chat room".to_string())
    })?;

    // Если это принудительный кик (а не самостоятельный выход), проверяем иерархию прав
    if requester_id != target_user_id {
        if req_role == "member" {
            return Err(ChatError::Forbidden(
                "Members cannot kick other users".to_string(),
            ));
        }
        if req_role == "moderator" && tgt_role != "member" {
            return Err(ChatError::Forbidden(
                "Moderators can only kick regular members".to_string(),
            ));
        }
        if tgt_role == "owner" {
            return Err(ChatError::Forbidden(
                "Cannot kick the owner of the room".to_string(),
            ));
        }
    }

    let mut tx = db.begin().await?;

    sqlx::query!(
        "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2",
        room_id,
        target_user_id
    )
    .execute(&mut *tx)
    .await?;

    // Проверяем количество оставшихся участников в этой комнате
    let remaining_count = sqlx::query_scalar!(
        "SELECT COUNT(*) as \"count!\" FROM room_members WHERE room_id = $1",
        room_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // Если участников не осталось, полностью удаляем комнату
    if remaining_count == 0 {
        sqlx::query!("DELETE FROM rooms WHERE id = $1", room_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub enum MessageDeletionResult {
    EveryoneDeleted { room_id: Uuid },
    MeDeleted,
}

/// Удаляет сообщение для всех (soft delete) или только скрывает для конкретного пользователя.
pub async fn delete_chat_message(
    db: &sqlx::PgPool,
    s3_client: &aws_sdk_s3::Client,
    requester_id: Uuid,
    message_id: Uuid,
    delete_type: &str,
) -> Result<MessageDeletionResult, ChatError> {
    if delete_type != "me" && delete_type != "everyone" {
        return Err(ChatError::BadRequest(
            "Invalid delete type. Must be 'me' or 'everyone'".to_string(),
        ));
    }

    // Проверяем существование сообщения, автора, роль запрашивающего и членство в комнате
    let msg_info = sqlx::query!(
        r#"
        SELECT 
            m.sender_id,
            m.room_id,
            (SELECT role::text FROM room_members rm WHERE rm.room_id = m.room_id AND rm.user_id = $2) as user_role,
            (SELECT EXISTS(SELECT 1 FROM room_members rm WHERE rm.room_id = m.room_id AND rm.user_id = $2)) as "is_member!"
        FROM messages m
        WHERE m.id = $1 AND m.deleted_at IS NULL
        "#,
        message_id,
        requester_id
    )
    .fetch_optional(db)
    .await?;

    let msg_info = msg_info.ok_or_else(|| ChatError::NotFound("Message not found".to_string()))?;

    if !msg_info.is_member {
        return Err(ChatError::Forbidden(
            "You do not have access to this room".to_string(),
        ));
    }

    if delete_type == "everyone" {
        let is_author = msg_info.sender_id == Some(requester_id);
        let is_privileged = msg_info.user_role.as_deref() == Some("owner")
            || msg_info.user_role.as_deref() == Some("moderator");

        // Удалять сообщения "у всех" могут только автор сообщения или владелец/модератор
        if !is_author && !is_privileged {
            return Err(ChatError::Forbidden(
                "You do not have permission to delete this message for everyone".to_string(),
            ));
        }

        let mut tx = db.begin().await?;

        sqlx::query!(
            "UPDATE messages SET deleted_at = NOW() WHERE id = $1",
            message_id
        )
        .execute(&mut *tx)
        .await?;

        // Запрашиваем вложения, чтобы потом удалить их из S3
        let attachments = sqlx::query!(
            "SELECT file_key FROM message_attachments WHERE message_id = $1",
            message_id
        )
        .fetch_all(&mut *tx)
        .await?;

        // Удаляем вложения из базы данных
        sqlx::query!(
            "DELETE FROM message_attachments WHERE message_id = $1",
            message_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Физическое удаление файлов из S3 (после фиксации транзакции)
        let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
            .unwrap_or_else(|_| "chat-files".to_string());
        for att in attachments {
            let _ = s3_client
                .delete_object()
                .bucket(&bucket)
                .key(&att.file_key)
                .send()
                .await;
        }

        Ok(MessageDeletionResult::EveryoneDeleted {
            room_id: msg_info.room_id,
        })
    } else {
        // Удаление "для себя" — просто скрываем сообщение в локальной таблице скрытых сообщений
        let mut tx = db.begin().await?;

        sqlx::query!(
            "INSERT INTO deleted_messages (message_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            message_id,
            requester_id
        )
        .execute(&mut *tx)
        .await?;

        let total_members = sqlx::query_scalar!(
            "SELECT COUNT(*) as \"count!\" FROM room_members WHERE room_id = $1",
            msg_info.room_id
        )
        .fetch_one(&mut *tx)
        .await?;

        let deleted_by_count = sqlx::query_scalar!(
            "SELECT COUNT(*) as \"count!\" FROM deleted_messages WHERE message_id = $1",
            message_id
        )
        .fetch_one(&mut *tx)
        .await?;

        let mut attachments_to_delete = Vec::new();
        let mut fully_deleted = false;

        // Если сообщение удалили все участники чата, полностью удаляем его из базы данных и S3
        if deleted_by_count >= total_members {
            fully_deleted = true;
            let attachments = sqlx::query!(
                "SELECT file_key FROM message_attachments WHERE message_id = $1",
                message_id
            )
            .fetch_all(&mut *tx)
            .await?;
            attachments_to_delete = attachments.into_iter().map(|a| a.file_key).collect();

            sqlx::query!(
                "DELETE FROM messages WHERE id = $1",
                message_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        if fully_deleted {
            let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
                .unwrap_or_else(|_| "chat-files".to_string());
            for key in attachments_to_delete {
                let _ = s3_client
                    .delete_object()
                    .bucket(&bucket)
                    .key(&key)
                    .send()
                    .await;
            }
        }

        Ok(MessageDeletionResult::MeDeleted)
    }
}

pub async fn handle_incoming_event(
    msg: &WsMessage,
    sender_id: &Uuid,
    chat_server: &crate::models::chat::ChatServerState,
    db: &sqlx::PgPool,
    s3_client: &aws_sdk_s3::Client,
) {
    match msg.event.as_str() {
        "send_message" => {
            if let Ok(payload) = serde_json::from_value::<SendMessagePayload>(msg.payload.clone()) {
                // Проверяем, состоит ли пользователь в этой комнате
                let is_member = sqlx::query_scalar!(
                    "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2) as \"exists!\"",
                    payload.room_id,
                    sender_id
                )
                .fetch_one(db)
                .await
                .unwrap_or(false);

                if !is_member {
                    return;
                }

                // Сохраняем сообщение и вложения в базу данных в рамках одной транзакции
                let msg_id = Uuid::new_v4();
                let created_at = chrono::Utc::now();

                let mut tx = match db.begin().await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Failed to start transaction for message save: {:?}", e);
                        return;
                    }
                };

                let db_result = sqlx::query!(
                    "INSERT INTO messages (id, room_id, sender_id, content, created_at) 
                     VALUES ($1, $2, $3, $4, $5)",
                    msg_id,
                    payload.room_id,
                    sender_id,
                    payload.content,
                    created_at
                )
                .execute(&mut *tx)
                .await;

                if db_result.is_err() {
                    let _ = tx.rollback().await;
                    return;
                }

                let mut saved_attachments = Vec::new();
                if let Some(attachments) = &payload.attachments {
                    let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
                        .unwrap_or_else(|_| "chat-files".to_string());
                    
                    for att in attachments {
                        let att_id = Uuid::new_v4();
                        let insert_result = sqlx::query!(
                            r#"
                            INSERT INTO message_attachments (id, message_id, file_key, file_name, file_size, file_mime)
                            VALUES ($1, $2, $3, $4, $5, $6)
                            "#,
                            att_id,
                            msg_id,
                            att.file_key,
                            att.file_name,
                            att.file_size,
                            att.file_mime
                        )
                        .execute(&mut *tx)
                        .await;

                        if let Err(e) = insert_result {
                            eprintln!("Failed to insert message attachment: {:?}", e);
                            let _ = tx.rollback().await;
                            return;
                        }

                        // Сразу генерируем presigned URL для рассылки события клиентам
                        let file_url = match crate::helpers::s3::get_presigned_download_url(
                            s3_client,
                            &bucket,
                            &att.file_key,
                            7200,
                        )
                        .await
                        {
                            Ok(url) => url,
                            Err(e) => {
                                eprintln!("Failed to generate presigned download URL for file_key {}: {:?}", att.file_key, e);
                                format!("{}/{}", bucket, att.file_key)
                            }
                        };

                        saved_attachments.push(AttachmentResponse {
                            id: att_id,
                            file_name: att.file_name.clone(),
                            file_size: att.file_size,
                            file_mime: att.file_mime.clone(),
                            file_url,
                        });
                    }
                }

                if tx.commit().await.is_ok() {
                    // Достаем из БД список всех участников этой комнаты, чтобы знать, кому слать уведомление
                    if let Ok(members) = sqlx::query!(
                        "SELECT user_id FROM room_members WHERE room_id = $1",
                        payload.room_id
                    )
                    .fetch_all(db)
                    .await
                    {
                        // Подготавливаем красивый JSON для фронта
                        let notification = crate::models::chat::NewMessageNotification {
                            id: msg_id,
                            room_id: payload.room_id,
                            sender_id: Some(*sender_id),
                            content: payload.content,
                            created_at,
                            attachments: if saved_attachments.is_empty() { None } else { Some(saved_attachments) },
                        };

                        let out_msg = WsMessage {
                            event: "new_message".to_string(),
                            payload: serde_json::to_value(notification).unwrap(),
                        };

                        // Рассылаем сообщение ВСЕМ участникам комнаты, кто сейчас онлайн
                        for member in members {
                            chat_server.send_to_user(&member.user_id, out_msg.clone());
                        }
                    }
                }
            }
        }

        "read_room" => {
            if let Ok(payload) = serde_json::from_value::<ReadRoomPayload>(msg.payload.clone()) {
                if let Ok(record) = sqlx::query!(
                    "UPDATE room_members SET last_read_at = NOW() 
                     WHERE room_id = $1 AND user_id = $2
                     RETURNING EXTRACT(EPOCH FROM last_read_at)::BIGINT as \"last_read_at!\"",
                    payload.room_id, sender_id
                )
                .fetch_one(db).await {
                    if let Ok(members) = sqlx::query!(
                        "SELECT user_id FROM room_members WHERE room_id = $1 AND user_id != $2",
                        payload.room_id, sender_id
                    )
                    .fetch_all(db)
                    .await {
                        let out_msg = WsMessage {
                            event: "room_read".to_string(),
                            payload: serde_json::json!({
                                "room_id": payload.room_id,
                                "user_id": sender_id,
                                "last_read_at": record.last_read_at
                            }),
                        };

                        for member in members {
                            chat_server.send_to_user(&member.user_id, out_msg.clone());
                        }
                    }
                }
            }
        }

        _ => println!("Неизвестное событие от клиента: {}", msg.event),
    }
}

pub async fn ws_session_loop(
    chat_server: Arc<crate::models::chat::ChatServerState>,
    db: sqlx::PgPool,
    s3_client: aws_sdk_s3::Client,
    user_id: Uuid,
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    mut rx: mpsc::UnboundedReceiver<WsMessage>,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            maybe_msg = msg_stream.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            handle_incoming_event(&ws_msg, &user_id, &chat_server, &db, &s3_client).await;
                        }
                    }
                    Some(Ok(Message::Close(reason))) => {
                        let _ = session.close(reason).await;
                        break;
                    }
                    Some(Ok(Message::Ping(bytes))) => {
                        let _ = session.pong(&bytes).await;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Nop)) => {}
                    _ => break,
                }
            }

            maybe_ws_msg = rx.recv() => {
                match maybe_ws_msg {
                    Some(ws_msg) => {
                        if let Ok(json_str) = serde_json::to_string(&ws_msg) {
                            if session.text(json_str).await.is_err() {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }

            _ = interval.tick() => {
                if session.ping(b"").await.is_err() {
                    break;
                }
            }
        }
    }

    if let Some(mut senders) = chat_server.sessions.get_mut(&user_id) {
        senders.retain(|tx| !tx.is_closed());
    }
}

pub struct PaginatedMessages {
    pub messages: Vec<ChatMessage>,
    pub total_count: i64,
    pub members: Vec<RoomMemberInfo>,
}

pub async fn get_room_messages_paginated(
    db: &sqlx::PgPool,
    s3_client: &aws_sdk_s3::Client,
    user_id: Uuid,
    room_id: Uuid,
    limit: i64,
    offset: i64,
    search_pattern: Option<String>,
) -> Result<PaginatedMessages, ChatError> {
    let is_member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2) as \"exists!\"",
        room_id,
        user_id
    )
    .fetch_one(db)
    .await
    .unwrap_or(false);

    if !is_member {
        return Err(ChatError::Forbidden(
            "You do not have access to this room".to_string(),
        ));
    }

    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM messages m
        WHERE m.room_id = $1
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM deleted_messages dm
              WHERE dm.message_id = m.id AND dm.user_id = $2
          )
          AND ($3::text IS NULL OR m.content ILIKE $3)
        "#,
        room_id,
        user_id,
        search_pattern
    )
    .fetch_one(db)
    .await?;

    let mut messages = sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT 
            m.id,
            m.room_id,
            m.sender_id,
            m.content,
            EXTRACT(EPOCH FROM m.created_at)::BIGINT as created_at,
            CASE 
                WHEN m.sender_id IS NOT NULL THEN json_build_object(
                    'id', u.id,
                    'username', u.username,
                    'first_name', u.first_name,
                    'last_name', u.last_name,
                    'avatar_url', u.avatar_url
                )
                ELSE NULL
            END as author,
            CASE
                WHEN m.sender_id = $2 THEN
                    EXISTS (
                        SELECT 1
                        FROM room_members rm_other
                        WHERE rm_other.room_id = m.room_id
                          AND rm_other.user_id != $2
                          AND rm_other.last_read_at >= m.created_at
                    )
                ELSE
                    EXISTS (
                        SELECT 1
                        FROM room_members rm_self
                        WHERE rm_self.room_id = m.room_id
                          AND rm_self.user_id = $2
                          AND rm_self.last_read_at >= m.created_at
                    )
            END as is_read
        FROM messages m
        LEFT JOIN users u ON m.sender_id = u.id
        WHERE m.room_id = $1
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM deleted_messages dm
              WHERE dm.message_id = m.id AND dm.user_id = $2
          )
          AND ($3::text IS NULL OR m.content ILIKE $3)
        ORDER BY m.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let mut message_attachments_map = std::collections::HashMap::new();

    if !message_ids.is_empty() {
        let attachments_raw = sqlx::query!(
            r#"
            SELECT id, message_id, file_key, file_name, file_size, file_mime
            FROM message_attachments
            WHERE message_id = ANY($1)
            "#,
            &message_ids
        )
        .fetch_all(db)
        .await?;

        let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
            .unwrap_or_else(|_| "chat-files".to_string());

        for att in attachments_raw {
            let file_url = match crate::helpers::s3::get_presigned_download_url(
                s3_client,
                &bucket,
                &att.file_key,
                7200,
            )
            .await
            {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("Failed to generate presigned download URL for file_key {}: {:?}", att.file_key, e);
                    format!("{}/{}", bucket, att.file_key)
                }
            };

            let response = AttachmentResponse {
                id: att.id,
                file_name: att.file_name,
                file_size: att.file_size,
                file_mime: att.file_mime,
                file_url,
            };

            message_attachments_map
                .entry(att.message_id)
                .or_insert_with(Vec::new)
                .push(response);
        }
    }

    for msg in &mut messages {
        msg.attachments = message_attachments_map.remove(&msg.id);
    }

    let members = sqlx::query_as::<_, RoomMemberInfo>(
        r#"
        SELECT 
            u.id,
            u.username,
            u.first_name,
            u.last_name,
            u.avatar_url,
            rm.role::text as role
        FROM room_members rm
        JOIN users u ON rm.user_id = u.id
        WHERE rm.room_id = $1
        "#,
    )
    .bind(room_id)
    .fetch_all(db)
    .await?;

    Ok(PaginatedMessages {
        messages,
        total_count,
        members,
    })
}

/// Проверяет URL на безопасность (защита от SSRF).
/// Разрешает только HTTP/HTTPS схемы и публичные IP-адреса.
pub fn is_url_safe(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    if !matches!(parsed.scheme(), "http" | "https") {
        return false;
    }

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    if host == "localhost" || host == "0.0.0.0" {
        return false;
    }

    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return !ip.is_loopback()
            && !ip.is_unspecified()
            && match ip {
                std::net::IpAddr::V4(v4) => {
                    !v4.is_private() && !v4.is_link_local() && !v4.is_broadcast()
                }
                std::net::IpAddr::V6(v6) => !v6.is_loopback(),
            };
    }

    true
}

/// Извлекает значение OG-метатега или другого атрибута из HTML по regex-паттерну.
pub fn extract_og(html: &str, pattern: &str) -> Option<String> {
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| html_escape::decode_html_entities(m.as_str()).into_owned())
}

