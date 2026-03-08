use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
    Idle,
    Working,
    Waiting,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeTier {
    Executive,
    Manager,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerStatus {
    Pending,
    Running,
    Stopped,
    Failed,
}

pub const BUILTIN_ROLES: &[&str] = &[
    "ceo",
    "cto",
    "cmo",
    "coo",
    "fullstack_dev",
    "frontend_dev",
    "backend_dev",
    "content_writer",
    "designer",
    "seo_specialist",
    "devops",
    "qa_engineer",
    "social_media",
    "data_analyst",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConfig {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub title: String,
    pub role: String,
    pub tier: EmployeeTier,
    pub manager_id: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub system_prompt: String,
    pub personality: Option<String>,
    pub tools: Vec<String>,
    pub config: serde_json::Value,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub manifest_id: Option<String>,
    pub container_status: Option<ContainerStatus>,
    pub worker_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeState {
    pub status: EmployeeStatus,
    pub current_task_id: Option<String>,
    pub last_active_at: DateTime<Utc>,
}
