use actix_web::{delete, get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

use crate::{
    helpers::{
        api::AuthenticatedUser,
        chats::{ChatError, MessageDeletionResult::EveryoneDeleted, create_direct_room, is_url_safe, extract_og},
    },
    models::{
        app::AppState,
        chat::{
            AddMemberRequest, ChatListItemResponse, ChatMessageResponse, ChatRoomType,
            CreateChatRoomRequest, DeleteMessageQuery, MessageDeletedNotification,
            ParseLinkQuery, RoomsParams, WsMessage,
        },
    },
};

#[get("/chats")]
pub async fn get_rooms(
    user: AuthenticatedUser,
    params: web::Query<RoomsParams>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let user_id = user.id;
    let limit = params.limit;

    let search_pattern = params
        .search_value
        .as_ref()
        .map(|val| format!("%{}%", val.trim()));

    let paginated = crate::helpers::chats::get_user_rooms_paginated(
        &state.pool,
        user_id,
        limit,
        params.offset(),
        search_pattern,
    )
    .await?;

    let total_pages = (paginated.total_count as f64 / limit as f64).ceil() as i64;

    let response: Vec<ChatListItemResponse> = paginated
        .rooms
        .into_iter()
        .map(|chat| {
            let last_msg_unwrapped = chat.last_message.map(|j| j.0);
            ChatListItemResponse {
                id: chat.id,
                room_type: chat.room_type,
                name: chat.name,
                description: chat.description,
                direct_key: chat.direct_key,
                created_at: chat.created_at,
                updated_at: chat.updated_at,
                unread_count: chat.unread_count,
                last_message: last_msg_unwrapped,
                direct_user: chat.direct_user.map(|j| j.0),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": response,
        "meta": {
            "current_page": params.page,
            "per_page": limit,
            "total_count": paginated.total_count,
            "total_pages": total_pages,
            "has_next": params.page < total_pages,
            "has_previous": params.page > 1
        }
    })))
}

/// Получает историю сообщений конкретного чата с поддержкой поиска, лимита и офсета.
#[get("/chats/{room_id}")]
pub async fn get_room_messages(
    user: AuthenticatedUser,
    room_id: web::Path<Uuid>,
    params: web::Query<RoomsParams>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let user_id = user.id;
    let room_id = room_id.into_inner();
    let limit = params.limit;

    // Подготовка ILIKE-паттерна для поиска сообщений
    let search_pattern = params
        .search_value
        .as_ref()
        .map(|val| format!("%{}%", val.trim()));

    // Загрузка истории сообщений чата через хелпер
    let paginated = crate::helpers::chats::get_room_messages_paginated(
        &state.pool,
        &state.s3_public_client,
        user_id,
        room_id,
        limit,
        params.offset(),
        search_pattern,
    )
    .await?;

    let total_pages = (paginated.total_count as f64 / limit as f64).ceil() as i64;

    // Приведение к ответной структуре с разворачиванием sqlx JSON-полей
    let response: Vec<ChatMessageResponse> = paginated
        .messages
        .into_iter()
        .map(|msg| {
            ChatMessageResponse {
                id: msg.id,
                room_id: msg.room_id,
                sender_id: msg.sender_id,
                content: msg.content,
                created_at: msg.created_at,
                author: msg.author.map(|j| j.0),
                is_read: msg.is_read,
                attachments: msg.attachments,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": response,
        "members": paginated.members,
        "meta": {
            "current_page": params.page,
            "per_page": limit,
            "total_count": paginated.total_count,
            "total_pages": total_pages,
            "has_next": params.page < total_pages,
            "has_previous": params.page > 1
        }
    })))
}


/// Создает новую комнату.
/// Поддерживает создание как личных (direct), так и групповых (group) чатов.
#[post("/chats")]
pub async fn create_room(
    user: AuthenticatedUser,
    params: web::Json<CreateChatRoomRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let author_id = user.id;

    params
        .validate()
        .map_err(|e| ChatError::Validation(e.to_string()))?;

    match params.room_type {
        ChatRoomType::Direct => {
            let user_2 = params.direct_user_id.ok_or_else(|| {
                ChatError::BadRequest("direct_user_id is required for direct chats".to_string())
            })?;

            let (room_id, already_exists) = create_direct_room(
                &state.pool,
                author_id,
                user_2,
            )
            .await?;
            
            if !already_exists {
                let ws_message = WsMessage {
                    event: "room_created".to_string(),
                    payload: serde_json::json!({ "room_id": room_id }),
                };

                state.chat_server.send_to_user(&author_id, ws_message.clone());
                state.chat_server.send_to_user(&author_id, ws_message);
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": {
                    "room_id": room_id,
                    "already_exists": already_exists
                }
            })))
        }
        ChatRoomType::Group => {
            let name = params
                .name
                .as_deref()
                .map(|n| n.trim())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    ChatError::BadRequest("name is required for group chats".to_string())
                })?;
            let description = params.description.as_deref();

            let room_id = crate::helpers::chats::create_group_room(
                &state.pool,
                author_id,
                name,
                description,
            )
            .await?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": {
                    "room_id": room_id
                }
            })))
        }
    }
}

/// Добавляет нового участника в групповой чат.
/// Проверяет права текущего пользователя (разрешено только owner и moderator).
#[post("/chats/{room_id}/members")]
pub async fn add_member(
    user: AuthenticatedUser,
    room_id: web::Path<Uuid>,
    params: web::Json<AddMemberRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let author_id = user.id;
    let room_id = room_id.into_inner();

    params
        .validate()
        .map_err(|e| ChatError::Validation(e.to_string()))?;

    crate::helpers::chats::add_room_member(
        &state.pool,
        author_id,
        room_id,
        params.user_id,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

/// Удаляет участника из группового чата.
/// Поддерживает как кик других пользователей модератором/владельцем, так и самостоятельный выход.
#[delete("/chats/{room_id}/members/{user_id}")]
pub async fn remove_member(
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let author_id = user.id;
    let (room_id, user_id) = path.into_inner();

    crate::helpers::chats::remove_room_member(
        &state.pool,
        author_id,
        room_id,
        user_id,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

/// Удаляет сообщение.
/// При type = 'everyone' сообщение помечается удаленным для всех (soft delete).
/// При type = 'me' сообщение скрывается только для текущего пользователя.
#[delete("/messages/{message_id}")]
pub async fn delete_message(
    user: AuthenticatedUser,
    message_id: web::Path<Uuid>,
    query: web::Query<DeleteMessageQuery>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let author_id = user.id;
    let message_id = message_id.into_inner();

    let result = crate::helpers::chats::delete_chat_message(
        &state.pool,
        &state.s3_client,
        author_id,
        message_id,
        &query.delete_type,
    )
    .await?;

    // Если удалили для всех, рассылаем уведомление по WebSocket
    if let EveryoneDeleted { room_id } = result {
        if let Ok(members) = sqlx::query!(
            "SELECT user_id FROM room_members WHERE room_id = $1",
            room_id
        )
        .fetch_all(&state.pool)
        .await
        {
            let notification = MessageDeletedNotification {
                message_id,
                room_id,
            };

            let ws_msg = WsMessage {
                event: "message_deleted".to_string(),
                payload: serde_json::to_value(notification).unwrap(),
            };

            for member in members {
                state
                    .chat_server
                    .send_to_user(&member.user_id, ws_msg.clone());
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}


use actix_multipart::Multipart;

#[derive(serde::Deserialize)]
pub struct UploadQuery {
    pub room_id: Uuid,
}

#[post("/chat/upload")]
pub async fn upload_file(
    user: AuthenticatedUser,
    query: web::Query<UploadQuery>,
    payload: Multipart,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    // Проверяем, состоит ли пользователь в этой комнате
    let is_member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2) as \"exists!\"",
        query.room_id,
        user.id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(ChatError::Database)?;

    if !is_member {
        return Err(ChatError::Forbidden("You do not have access to this room".to_string()));
    }

    // Лимит 300 МБ
    let max_size = 300 * 1024 * 1024;
    let file = crate::helpers::images::read_multipart_file(payload, max_size)
        .await
        .map_err(|e| ChatError::BadRequest(e.to_string()))?;

    let mut filename = file.filename;
    let mut mime = file.content_type;
    let mut bytes = file.bytes;

    if mime.starts_with("image/") {
        match crate::helpers::images::resize_and_encode_webp(&bytes, 1600, 1600) {
            Ok(webp_bytes) => {
                bytes = webp_bytes;
                mime = "image/webp".to_string();
                let path = std::path::Path::new(&filename);
                if let Some(stem) = path.file_stem() {
                    filename = format!("{}.webp", stem.to_string_lossy());
                } else {
                    filename = "image.webp".to_string();
                }
            }
            Err(e) => {
                eprintln!("Failed to optimize image, uploading original: {}", e);
            }
        }
    }

    let bucket_name = std::env::var("MINIO_BUCKET_CHAT_FILES")
        .unwrap_or_else(|_| "chat-files".to_string());

    // Уникальный ключ по пути: chats/{room_id}/{uuid}_{filename}
    let key = format!("chats/{}/{}_{}", query.room_id, uuid::Uuid::new_v4(), filename);
    let file_size = bytes.len() as i64;

    state
        .s3_client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(bytes.into())
        .content_type(&mime)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to upload chat file to S3: {:?}", e);
            ChatError::Database(sqlx::Error::Protocol("S3 upload failed".to_string()))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": {
            "file_key": key,
            "file_name": filename,
            "file_mime": mime,
            "file_size": file_size
        }
    })))
}


#[get("/chats/parse-link")]
pub async fn parse_link(
    query: web::Query<ParseLinkQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let url = query.url.trim().to_string();

    if !is_url_safe(&url) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Invalid or disallowed URL"
        }));
    }

    let response = state
        .client
        .get(&url)
        .timeout(std::time::Duration::from_secs(4))
        .header("User-Agent", "Mozilla/5.0 (compatible; AgoraBot/1.0; +https://agora.ru)")
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await;

    let resp = match response {
        Ok(r) => r,
        Err(e) => {
            eprintln!("parse-link fetch error for {}: {}", url, e);
            return HttpResponse::BadGateway().json(serde_json::json!({
                "status": "error",
                "message": "Failed to fetch URL"
            }));
        }
    };

    // Читаем не более 256 КБ, чтобы не тащить тяжёлые страницы целиком
    const MAX_BYTES: usize = 256 * 1024;
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadGateway().json(serde_json::json!({
                "status": "error",
                "message": "Failed to read response body"
            }));
        }
    };
    let html = String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_BYTES)]);


    let title = extract_og(
        &html,
        r#"(?i)<meta[^>]+property=["']og:title["'][^>]+content=["']([^"']+)["']"#,
    )
    .or_else(|| {
        extract_og(
            &html,
            r#"(?i)<meta[^>]+content=["']([^"']+)["'][^>]+property=["']og:title["']"#,
        )
    })
    .or_else(|| extract_og(&html, r#"(?i)<title[^>]*>([^<]+)</title>"#));

    let title = match title {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return HttpResponse::Ok().json(serde_json::json!({
                "status": "error",
                "message": "No title found"
            }));
        }
    };

    let description = extract_og(
        &html,
        r#"(?i)<meta[^>]+property=["']og:description["'][^>]+content=["']([^"']+)["']"#,
    )
    .or_else(|| {
        extract_og(
            &html,
            r#"(?i)<meta[^>]+content=["']([^"']+)["'][^>]+property=["']og:description["']"#,
        )
    })
    .or_else(|| {
        // Фоллбэк на meta description
        extract_og(
            &html,
            r#"(?i)<meta[^>]+name=["']description["'][^>]+content=["']([^"']+)["']"#,
        )
    });

    let image_url = extract_og(
        &html,
        r#"(?i)<meta[^>]+property=["']og:image["'][^>]+content=["']([^"']+)["']"#,
    )
    .or_else(|| {
        extract_og(
            &html,
            r#"(?i)<meta[^>]+content=["']([^"']+)["'][^>]+property=["']og:image["']"#,
        )
    });

    HttpResponse::Ok().json(serde_json::json!({
        "url": url,
        "title": title,
        "description": description,
        "image_url": image_url,
    }))
}