use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyPlan {
    pub name: String,
    pub description: String,
    pub product: ProductSpec,
    pub team: Vec<TeamMember>,
    pub milestones: Vec<Milestone>,
    pub estimated_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpec {
    #[serde(rename = "type")]
    pub product_type: String,
    pub tech_stack: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub role: String,
    pub title: String,
    pub name: String,
    pub responsibilities: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub name: String,
    pub tasks: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyPhase {
    Planning,
    PlanReady,
    Approved,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRecord {
    pub id: String,
    pub description: String,
    pub phase: CompanyPhase,
    pub plan: Option<CompanyPlan>,
    pub ceo_employee_id: Option<String>,
    pub employees: Vec<CompanyEmployee>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyStatus {
    pub company_id: String,
    pub name: String,
    pub status: String,
    pub employees: Vec<CompanyEmployee>,
    pub tasks_completed: u32,
    pub tasks_pending: u32,
    pub total_cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyEmployee {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub current_task: Option<String>,
}
