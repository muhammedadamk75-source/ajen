use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Fact,
    Decision,
    Context,
    Preference,
    Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryImportance {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub company_id: String,
    pub employee_id: Option<String>,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub importance: MemoryImportance,
    pub created_at: DateTime<Utc>,
}
