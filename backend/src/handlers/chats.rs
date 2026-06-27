use actix_web::{delete, get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

use crate::{
    helpers::{api::AuthenticatedUser, chats::{ChatError, MessageDeletionResult::EveryoneDeleted, create_direct_room}}, models::{
        app::AppState, chat::{
            AddMemberRequest, ChatListItemResponse, ChatMessageResponse, ChatRoomType, CreateChatRoomRequest, DeleteMessageQuery, MessageDeletedNotification, RoomsParams, WsMessage,
        },
    },
};

/// Получает пагинированный список чат-комнат пользователя.
/// Поддерживает фильтрацию по поисковой строке `search_value`.
#[get("/chats")]
pub async fn get_rooms(
    user: AuthenticatedUser,
    params: web::Query<RoomsParams>,
    state: web::Data<AppState>,
) -> Result<impl Responder, ChatError> {
    let user_id = user.id;
    let limit = params.limit;

    // Подготовка ILIKE-паттерна для поиска
    let search_pattern = params
        .search_value
        .as_ref()
        .map(|val| format!("%{}%", val.trim()));

    // Загрузка комнат из БД
    let paginated = crate::helpers::chats::get_user_rooms_paginated(
        &state.pool,
        user_id,
        limit,
        params.offset(),
        search_pattern,
    )
    .await?;

    let total_pages = (paginated.total_count as f64 / limit as f64).ceil() as i64;

    // Приведение к ответной структуре с разворачиванием sqlx JSON полей
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
