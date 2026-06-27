use actix_ws::Message;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use actix_web::{ResponseError, HttpResponse, http::StatusCode};
use std::fmt;
use crate::models::chat::{WsMessage, SendMessagePayload, ReadRoomPayload, ChatListItem};

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

/// Получает пагинированный список комнат пользователя с поддержкой поиска.
pub async fn get_user_rooms_paginated(
    db: &sqlx::PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
    search_pattern: Option<String>,
) -> Result<PaginatedRooms, ChatError> {
    // 1. Считаем общее количество подходящих комнат для метаданных пагинации
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

    // 2. Получаем пагинированный список комнат вместе с последним сообщением и информацией о собеседнике
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
                    )
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

/// Создает личную комнату (direct chat) между двумя пользователями.
/// Возвращает UUID комнаты и флаг `already_exists`, если комната была создана ранее.
pub async fn create_direct_room(
    db: &sqlx::PgPool,
    author_id: Uuid,
    user_2: Uuid,
) -> Result<(Uuid, bool), ChatError> {
    if author_id == user_2 {
        return Err(ChatError::BadRequest("Cannot create direct chat with yourself".to_string()));
    }

    // Алфавитный ключ для уникальности переписки двух пользователей
    let (min_user, max_user) = if author_id < user_2 {
        (author_id, user_2)
    } else {
        (user_2, author_id)
    };
    let direct_key = format!("{}:{}", min_user, max_user);

    // Проверяем, существует ли уже такой чат
    let existing_room = sqlx::query_scalar!(
        "SELECT id FROM rooms WHERE direct_key = $1",
        direct_key
    )
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

/// Добавляет нового участника в групповой чат, проверяя права запрашивающего.
pub async fn add_room_member(
    db: &sqlx::PgPool,
    requester_id: Uuid,
    room_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), ChatError> {
    // 1. Проверяем тип комнаты и роль запрашивающего
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
        return Err(ChatError::BadRequest("Cannot add members to direct chats".to_string()));
    }

    let role = info.requester_role.ok_or_else(|| {
        ChatError::Forbidden("You are not a member of this chat room".to_string())
    })?;

    // Приглашать могут только владельцы и модераторы
    if role != "owner" && role != "moderator" {
        return Err(ChatError::Forbidden("You do not have permission to invite members (must be owner or moderator)".to_string()));
    }

    // 2. Проверяем существование приглашаемого пользователя
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

    // 3. Добавляем в участники
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
        Err(ChatError::Conflict("User is already a member of this room".to_string()))
    } else {
        Ok(())
    }
}

/// Удаляет участника из группового чата (кик или самостоятельный выход), проверяя права.
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

    let roles_info = roles.ok_or_else(|| ChatError::NotFound("Room or member information not found".to_string()))?;

    if roles_info.room_type.as_deref() == Some("direct") {
        return Err(ChatError::BadRequest("Cannot modify members in direct chats".to_string()));
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
            return Err(ChatError::Forbidden("Members cannot kick other users".to_string()));
        }
        if req_role == "moderator" && tgt_role != "member" {
            return Err(ChatError::Forbidden("Moderators can only kick regular members".to_string()));
        }
        if tgt_role == "owner" {
            return Err(ChatError::Forbidden("Cannot kick the owner of the room".to_string()));
        }
    }

    sqlx::query!(
        "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2",
        room_id,
        target_user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub enum MessageDeletionResult {
    EveryoneDeleted { room_id: Uuid },
    MeDeleted,
}

/// Удаляет сообщение для всех (soft delete) или только скрывает для конкретного пользователя.
pub async fn delete_chat_message(
    db: &sqlx::PgPool,
    requester_id: Uuid,
    message_id: Uuid,
    delete_type: &str,
) -> Result<MessageDeletionResult, ChatError> {
    if delete_type != "me" && delete_type != "everyone" {
        return Err(ChatError::BadRequest("Invalid delete type. Must be 'me' or 'everyone'".to_string()));
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
        return Err(ChatError::Forbidden("You do not have access to this room".to_string()));
    }

    if delete_type == "everyone" {
        let is_author = msg_info.sender_id == Some(requester_id);
        let is_privileged = msg_info.user_role.as_deref() == Some("owner")
            || msg_info.user_role.as_deref() == Some("moderator");

        // Удалять сообщения "у всех" могут только автор сообщения или владелец/модератор
        if !is_author && !is_privileged {
            return Err(ChatError::Forbidden("You do not have permission to delete this message for everyone".to_string()));
        }

        sqlx::query!(
            "UPDATE messages SET deleted_at = NOW() WHERE id = $1",
            message_id
        )
        .execute(db)
        .await?;

        Ok(MessageDeletionResult::EveryoneDeleted { room_id: msg_info.room_id })
    } else {
        // Удаление "для себя" — просто скрываем сообщение в локальной таблице скрытых сообщений
        sqlx::query!(
            "INSERT INTO deleted_messages (message_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            message_id,
            requester_id
        )
        .execute(db)
        .await?;

        Ok(MessageDeletionResult::MeDeleted)
    }
}

pub async fn handle_incoming_event(
    msg: &WsMessage,
    sender_id: &Uuid,
    chat_server: &crate::models::chat::ChatServerState,
    db: &sqlx::PgPool,
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

                // Сохраняем сообщение в базу данных PostgreSQL через SQLx
                let msg_id = Uuid::new_v4();
                let created_at = chrono::Utc::now();
                
                let db_result = sqlx::query!(
                    "INSERT INTO messages (id, room_id, sender_id, content, created_at) 
                     VALUES ($1, $2, $3, $4, $5)",
                    msg_id, payload.room_id, sender_id, payload.content, created_at
                )
                .execute(db).await;

                if db_result.is_ok() {
                    // 3. Достаем из БД список всех участников этой комнаты, чтобы знать, кому слать уведомление
                    if let Ok(members) = sqlx::query!(
                        "SELECT user_id FROM room_members WHERE room_id = $1",
                        payload.room_id
                    ).fetch_all(db).await {

                        // Подготавливаем красивый JSON для фронта
                        let notification = crate::models::chat::NewMessageNotification {
                            id: msg_id,
                            room_id: payload.room_id,
                            sender_id: Some(*sender_id),
                            content: payload.content,
                            created_at,
                        };

                        let out_msg = WsMessage {
                            event: "new_message".to_string(),
                            payload: serde_json::to_value(notification).unwrap(),
                        };

                        // 4. Рассылаем сообщение ВСЕМ участникам комнаты, кто сейчас онлайн
                        for member in members {
                            chat_server.send_to_user(&member.user_id, out_msg.clone());
                        }
                    }
                }
            }
        }

        "read_room" => {
            if let Ok(payload) = serde_json::from_value::<ReadRoomPayload>(msg.payload.clone()) {
                // Обновляем метку времени прочтения в БД
                let _ = sqlx::query!(
                    "UPDATE room_members SET last_read_at = NOW() WHERE room_id = $1 AND user_id = $2",
                    payload.room_id, sender_id
                )
                .execute(db).await;
            }
        }

        _ => println!("Неизвестное событие от клиента: {}", msg.event),
    }
}

pub async fn ws_session_loop(
    chat_server: Arc<crate::models::chat::ChatServerState>,
    db: sqlx::PgPool,
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
                            handle_incoming_event(&ws_msg, &user_id, &chat_server, &db).await;
                        }
                    }
                    Some(Ok(Message::Close(reason))) => {
                        let _ = session.close(reason).await;
                        break;
                    }
                    Some(Ok(Message::Pong(_))) => {}
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
