use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,        // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSessionListItem {
    pub id: String,
    pub title: String,
    pub message_count: usize,
    pub updated_at: DateTime<Utc>,
}

impl From<&ChatSession> for ChatSessionListItem {
    fn from(session: &ChatSession) -> Self {
        Self {
            id: session.id.clone(),
            title: session.title.clone(),
            message_count: session.messages.len(),
            updated_at: session.updated_at,
        }
    }
}
