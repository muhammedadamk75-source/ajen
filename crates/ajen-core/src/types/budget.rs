use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetConfig {
    pub company_limit_cents: Option<i64>,
    pub per_employee_limit_cents: Option<i64>,
    pub warning_threshold_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetCheck {
    pub allowed: bool,
    pub used_cents: i64,
    pub limit_cents: Option<i64>,
    pub percent_used: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub total_cost_cents: i64,
    pub by_employee: HashMap<String, i64>,
    pub by_model: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    pub company_id: String,
    pub employee_id: String,
    pub task_id: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub cost_cents: i64,
}
