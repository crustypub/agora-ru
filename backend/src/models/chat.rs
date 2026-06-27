use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use tokio::sync::mpsc;
use uuid::Uuid;
use validator::Validate;
use crate::models::app::{Author, default_limit, default_page};
use sqlx::{types::Json};


#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Chat {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub direct_key: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct RoomsParams {
    pub search_value: Option<String>,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_limit")]
    pub limit: i64,
}

impl RoomsParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ChatListItem {
    pub id: Uuid,
    pub room_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub direct_key: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub unread_count: i64,
    pub last_message: Option<Json<ChatMessage>>,
    pub direct_user: Option<Json<Author>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatListItemResponse {
    pub id: Uuid,
    pub room_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub direct_key: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub unread_count: i64,
    pub last_message: Option<ChatMessage>,
    pub direct_user: Option<Author>,
}

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

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub created_at: i64,
    pub author: Option<Json<Author>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub created_at: i64,
    pub author: Option<Json<Author>>,
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

