use actix_ws::Message;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::models::chat::{WsMessage, SendMessagePayload, ReadRoomPayload};

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
