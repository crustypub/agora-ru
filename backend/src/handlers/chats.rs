use actix_web::{delete, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

use crate::{
    helpers::api::AuthenticatedUser,
    models::{
        app::AppState,
        chat::{AddMemberRequest, ChatRoomType, CreateChatRoomRequest, DeleteMessageQuery, MessageDeletedNotification, WsMessage},
    },
};

#[post("/chats")]
pub async fn create_room(
    user: AuthenticatedUser,
    params: web::Json<CreateChatRoomRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    match params.room_type {
        ChatRoomType::Direct => {
            let user_2 = match params.direct_user_id {
                Some(id) => id,
                None => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "direct_user_id is required for direct chats" })),
            };

            if author_id == user_2 {
                return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Cannot create direct chat with yourself" }));
            }

            let (min_user, max_user) = if author_id < user_2 { (author_id, user_2) } else { (user_2, author_id) };
            let direct_key = format!("{}:{}", min_user, max_user);

            let mut tx = match state.pool.begin().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Failed to start transaction: {}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database error" }));
                }
            };

            let room_id = match sqlx::query_scalar!(
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
            .await {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to insert or update room: {}", e);
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create chat room" }));
                }
            };

            if let Err(e) = sqlx::query!(
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
            .await {
                eprintln!("Failed to insert room member 1: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to add user to chat room" }));
            }

            if let Err(e) = sqlx::query!(
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
            .await {
                eprintln!("Failed to insert room member 2: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to add direct partner to chat room" }));
            }

            if let Err(e) = tx.commit().await {
                eprintln!("Failed to commit transaction: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database transaction commit failed" }));
            }

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": {
                    "room_id": room_id
                }
            }))
        }
        ChatRoomType::Group => {
            let name = match &params.name {
                Some(n) if !n.trim().is_empty() => n.trim(),
                _ => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "name is required for group chats" })),
            };
            let description = params.description.as_deref().unwrap_or("");

            let mut tx = match state.pool.begin().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Failed to start transaction: {}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database error" }));
                }
            };

            let room_id = match sqlx::query_scalar!(
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
            .await {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to create group room: {}", e);
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create group chat room" }));
                }
            };

            if let Err(e) = sqlx::query!(
                r#"
                INSERT INTO room_members (room_id, user_id, role)
                VALUES ($1, $2, $3::room_role)
                "#,
                room_id,
                author_id,
                "owner" as &str
            )
            .execute(&mut *tx)
            .await {
                eprintln!("Failed to add owner: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to set room owner" }));
            }

            if let Err(e) = tx.commit().await {
                eprintln!("Failed to commit transaction: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database transaction commit failed" }));
            }

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": {
                    "room_id": room_id
                }
            }))
        }
    }
}

#[post("/chats/{room_id}/members")]
pub async fn add_member(
    user: AuthenticatedUser,
    room_id: web::Path<Uuid>,
    params: web::Json<AddMemberRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;
    let room_id = room_id.into_inner();

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    let room_info = sqlx::query!(
        r#"
        SELECT type::text as room_type,
               (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role
        FROM rooms WHERE id = $1
        "#,
        room_id,
        author_id
    )
    .fetch_optional(&state.pool)
    .await;

    let info = match room_info {
        Ok(Some(info)) => info,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Room not found" })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database error" }));
        }
    };

    if info.room_type.as_deref() == Some("direct") {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Cannot add members to direct chats" }));
    }

    let role = match info.requester_role {
        Some(r) => r,
        None => return HttpResponse::Forbidden().json(serde_json::json!({ "error": "You are not a member of this chat room" })),
    };

    if role != "owner" && role != "moderator" {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": "You do not have permission to invite members (must be owner or moderator)" }));
    }

    let user_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as \"exists!\"",
        params.user_id
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if !user_exists {
        return HttpResponse::NotFound().json(serde_json::json!({ "error": "User to invite not found" }));
    }

    let insert_res = sqlx::query!(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
        room_id,
        params.user_id,
        "member" as &str
    )
    .execute(&state.pool)
    .await;

    match insert_res {
        Ok(result) => {
            if result.rows_affected() == 0 {
                HttpResponse::Conflict().json(serde_json::json!({ "error": "User is already a member of this room" }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({ "status": "success" }))
            }
        }
        Err(e) => {
            eprintln!("Database error adding member: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to add member" }))
        }
    }
}

#[delete("/chats/{room_id}/members/{user_id}")]
pub async fn remove_member(
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;
    let (room_id, user_id) = path.into_inner();

    let roles = sqlx::query!(
        r#"
        SELECT 
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role,
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $3) as target_role,
            (SELECT type::text FROM rooms WHERE id = $1) as room_type
        "#,
        room_id,
        author_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    let roles_info = match roles {
        Ok(Some(info)) => info,
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Room or member information not found" })),
    };

    if roles_info.room_type.as_deref() == Some("direct") {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Cannot modify members in direct chats" }));
    }

    let req_role = match roles_info.requester_role {
        Some(r) => r,
        None => return HttpResponse::Forbidden().json(serde_json::json!({ "error": "You are not a member of this chat room" })),
    };

    let tgt_role = match roles_info.target_role {
        Some(r) => r,
        None => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Target user is not a member of this chat room" })),
    };

    if author_id != user_id {
        if req_role == "member" {
            return HttpResponse::Forbidden().json(serde_json::json!({ "error": "Members cannot kick other users" }));
        }
        if req_role == "moderator" && tgt_role != "member" {
            return HttpResponse::Forbidden().json(serde_json::json!({ "error": "Moderators can only kick regular members" }));
        }
        if tgt_role == "owner" {
            return HttpResponse::Forbidden().json(serde_json::json!({ "error": "Cannot kick the owner of the room" }));
        }
    }

    let delete_res = sqlx::query!(
        "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2",
        room_id,
        user_id
    )
    .execute(&state.pool)
    .await;

    match delete_res {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => {
            eprintln!("Database error removing member: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to remove member" }))
        }
    }
}

#[delete("/messages/{message_id}")]
pub async fn delete_message(
    user: AuthenticatedUser,
    message_id: web::Path<Uuid>,
    query: web::Query<DeleteMessageQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;
    let message_id = message_id.into_inner();

    if query.delete_type != "me" && query.delete_type != "everyone" {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid delete type. Must be 'me' or 'everyone'" }));
    }

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
        author_id
    )
    .fetch_optional(&state.pool)
    .await;

    let msg_info = match msg_info {
        Ok(Some(info)) => info,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Message not found" })),
        Err(e) => {
            eprintln!("Database error checking message: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database error" }));
        }
    };

    if !msg_info.is_member {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": "You do not have access to this room" }));
    }

    if query.delete_type == "everyone" {
        let is_author = msg_info.sender_id == Some(author_id);
        let is_privileged = msg_info.user_role.as_deref() == Some("owner") || msg_info.user_role.as_deref() == Some("moderator");

        if !is_author && !is_privileged {
            return HttpResponse::Forbidden().json(serde_json::json!({ "error": "You do not have permission to delete this message for everyone" }));
        }

        let update_res = sqlx::query!(
            "UPDATE messages SET deleted_at = NOW() WHERE id = $1",
            message_id
        )
        .execute(&state.pool)
        .await;

        match update_res {
            Ok(_) => {
                if let Ok(members) = sqlx::query!(
                    "SELECT user_id FROM room_members WHERE room_id = $1",
                    msg_info.room_id
                )
                .fetch_all(&state.pool)
                .await {
                    let notification = MessageDeletedNotification {
                        message_id,
                        room_id: msg_info.room_id,
                    };

                    let ws_msg = WsMessage {
                        event: "message_deleted".to_string(),
                        payload: serde_json::to_value(notification).unwrap(),
                    };

                    for member in members {
                        state.chat_server.send_to_user(&member.user_id, ws_msg.clone());
                    }
                }

                HttpResponse::Ok().json(serde_json::json!({ "status": "success" }))
            }
            Err(e) => {
                eprintln!("Database error deleting message: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to delete message" }))
            }
        }
    } else {
        let insert_res = sqlx::query!(
            "INSERT INTO deleted_messages (message_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            message_id,
            author_id
        )
        .execute(&state.pool)
        .await;

        match insert_res {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
            Err(e) => {
                eprintln!("Database error hiding message: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to hide message" }))
            }
        }
    }
}
