use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use tokio::sync::mpsc;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize)]
pub enum ChatRoomType {
    Direct,
    Group,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsMessage {
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub room_id: Uuid,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ReadRoomPayload {
    pub room_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct NewMessageNotification {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageDeletedNotification {
    pub message_id: Uuid,
    pub room_id: Uuid,
}

type Tx = mpsc::UnboundedSender<WsMessage>;

pub struct ChatServerState {
    pub sessions: DashMap<Uuid, Vec<Tx>>,
}

impl ChatServerState {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Метод для отправки сообщения конкретному пользователю (на все его вкладки)
    pub fn send_to_user(&self, user_id: &Uuid, message: WsMessage) {
        if let Some(mut senders) = self.sessions.get_mut(user_id) {
            senders.retain(|tx| tx.send(message.clone()).is_ok());
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChatRoomRequest {
    pub room_type: ChatRoomType,

    pub direct_user_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMessageQuery {
    #[serde(rename = "type")]
    pub delete_type: String,
}

