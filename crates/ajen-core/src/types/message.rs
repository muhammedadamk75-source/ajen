use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Directive,
    Report,
    Request,
    Response,
    Broadcast,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub company_id: String,
    pub from_id: Option<String>,
    pub to_id: Option<String>,
    pub task_id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
