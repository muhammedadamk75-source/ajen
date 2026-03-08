use serde::{Deserialize, Serialize};

use super::employee::EmployeeTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolContext {
    pub company_id: String,
    pub employee_id: String,
    pub task_id: Option<String>,
    pub work_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: serde_json::Value,
    pub allowed_tiers: Vec<EmployeeTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
}
