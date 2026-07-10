use crate::db::chats::{
    check_user_exists,
    count_room_messages, create_direct_room_transaction, create_group_room_transaction,
    delete_message_everyone_transaction, delete_message_me_transaction,
    get_direct_room_by_key, get_member_roles_info, get_message_attachments_raw,
    get_message_info_for_deletion, get_other_room_member_ids, get_room_members_info,
    get_room_messages, get_room_requester_info, get_user_rooms, get_user_rooms_count,
    insert_room_member, remove_room_member_transaction, save_message_and_attachments_transaction,
    update_last_read_at, get_room_member_ids, check_is_room_member,
};
use crate::models::chat::{
    AttachmentResponse, ChatListItem, ChatMessage, ReadRoomPayload, RoomMemberInfo,
    SendMessagePayload, WsMessage,
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
    let search_ref = search_pattern.as_deref();
    let total_count = get_user_rooms_count(db, user_id, search_ref).await?;
    let rooms = get_user_rooms(db, user_id, limit, offset, search_ref).await?;

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
    let existing_room = get_direct_room_by_key(db, &direct_key).await?;

    if let Some(room_id) = existing_room {
        return Ok((room_id, true));
    }

    let room_id = create_direct_room_transaction(db, author_id, user_2, &direct_key).await?;

    Ok((room_id, false))
}

/// Создает групповую комнату и делает создателя ее владельцем (owner).
pub async fn create_group_room(
    db: &sqlx::PgPool,
    author_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Uuid, ChatError> {
    let room_id = create_group_room_transaction(db, author_id, name, description).await?;
    Ok(room_id)
}

pub async fn add_room_member(
    db: &sqlx::PgPool,
    requester_id: Uuid,
    room_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), ChatError> {
    let room_info = get_room_requester_info(db, room_id, requester_id).await?;

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

    let user_exists = check_user_exists(db, target_user_id).await.unwrap_or(false);

    if !user_exists {
        return Err(ChatError::NotFound("User to invite not found".to_string()));
    }

    let rows_affected = insert_room_member(db, room_id, target_user_id, "member").await?;

    if rows_affected == 0 {
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
    let roles = get_member_roles_info(db, room_id, requester_id, target_user_id).await?;

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

    remove_room_member_transaction(db, room_id, target_user_id).await?;

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
    let msg_info = get_message_info_for_deletion(db, message_id, requester_id).await?;

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

        let file_keys_to_delete = delete_message_everyone_transaction(db, message_id).await?;

        // Физическое удаление файлов из S3 (после фиксации транзакции)
        let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
            .unwrap_or_else(|_| "chat-files".to_string());
        for file_key in file_keys_to_delete {
            let _ = s3_client
                .delete_object()
                .bucket(&bucket)
                .key(&file_key)
                .send()
                .await;
        }

        Ok(MessageDeletionResult::EveryoneDeleted {
            room_id: msg_info.room_id,
        })
    } else {
        // Удаление "для себя"
        let (fully_deleted, attachments_to_delete) = delete_message_me_transaction(db, message_id, requester_id, msg_info.room_id).await?;

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
                let is_member = check_is_room_member(db, payload.room_id, *sender_id)
                    .await
                    .unwrap_or(false);

                if !is_member {
                    return;
                }

                // Сохраняем сообщение и вложения в базу данных в рамках одной транзакции
                let msg_id = Uuid::new_v4();
                let created_at = chrono::Utc::now();

                let save_res = save_message_and_attachments_transaction(
                    db,
                    msg_id,
                    payload.room_id,
                    *sender_id,
                    &payload.content,
                    created_at,
                    &payload.attachments,
                )
                .await;

                if save_res.is_err() {
                    return;
                }

                let mut saved_attachments = Vec::new();
                if payload.attachments.is_some() {
                    let bucket = std::env::var("MINIO_BUCKET_CHAT_FILES")
                        .unwrap_or_else(|_| "chat-files".to_string());
                    
                    // Нам нужно знать id вложений. Поскольку в транзакции мы их создали,
                    // мы можем сделать запрос по message_id или сгенерировать их заранее.
                    // Для простоты, так как мы только что сохранили их, мы можем
                    // запросить их из message_attachments:
                    if let Ok(atts_from_db) = get_message_attachments_raw(db, &[msg_id]).await {
                        for att in atts_from_db {
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
                                id: att.id,
                                file_name: att.file_name,
                                file_size: att.file_size,
                                file_mime: att.file_mime,
                                file_url,
                            });
                        }
                    }
                }

                // Достаем из БД список всех участников этой комнаты, чтобы знать, кому слать уведомление
                if let Ok(members) = get_room_member_ids(db, payload.room_id).await {
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
                    for member_id in members {
                        chat_server.send_to_user(&member_id, out_msg.clone());
                    }
                }
            }
        }

        "read_room" => {
            if let Ok(payload) = serde_json::from_value::<ReadRoomPayload>(msg.payload.clone()) {
                if let Ok(last_read_at) = update_last_read_at(db, payload.room_id, *sender_id).await {
                    if let Ok(members) = get_other_room_member_ids(db, payload.room_id, *sender_id).await {
                        let out_msg = WsMessage {
                            event: "room_read".to_string(),
                            payload: serde_json::json!({
                                "room_id": payload.room_id,
                                "user_id": sender_id,
                                "last_read_at": last_read_at
                            }),
                        };

                        for member_id in members {
                            chat_server.send_to_user(&member_id, out_msg.clone());
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
    let search_ref = search_pattern.as_deref();
    let is_member = check_is_room_member(db, room_id, user_id).await.unwrap_or(false);

    if !is_member {
        return Err(ChatError::Forbidden(
            "You do not have access to this room".to_string(),
        ));
    }

    let total_count = count_room_messages(db, room_id, user_id, search_ref).await?;
    let mut messages = get_room_messages(db, room_id, user_id, search_ref, limit, offset).await?;

    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let mut message_attachments_map = std::collections::HashMap::new();

    if !message_ids.is_empty() {
        let attachments_raw = get_message_attachments_raw(db, &message_ids).await?;

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

    let members = get_room_members_info(db, room_id).await?;

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
