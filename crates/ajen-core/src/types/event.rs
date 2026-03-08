use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    // Employee lifecycle
    #[serde(rename = "employee.spawned")]
    EmployeeSpawned,
    #[serde(rename = "employee.started")]
    EmployeeStarted,
    #[serde(rename = "employee.idle")]
    EmployeeIdle,
    #[serde(rename = "employee.terminated")]
    EmployeeTerminated,

    // Task lifecycle
    #[serde(rename = "task.created")]
    TaskCreated,
    #[serde(rename = "task.assigned")]
    TaskAssigned,
    #[serde(rename = "task.started")]
    TaskStarted,
    #[serde(rename = "task.completed")]
    TaskCompleted,
    #[serde(rename = "task.failed")]
    TaskFailed,

    // Tool usage
    #[serde(rename = "tool.called")]
    ToolCalled,
    #[serde(rename = "tool.completed")]
    ToolCompleted,
    #[serde(rename = "tool.failed")]
    ToolFailed,

    // LLM interactions
    #[serde(rename = "llm.request")]
    LlmRequest,
    #[serde(rename = "llm.response")]
    LlmResponse,

    // Communication
    #[serde(rename = "comms.sent")]
    CommsSent,
    #[serde(rename = "comms.received")]
    CommsReceived,

    // Company lifecycle
    #[serde(rename = "company.created")]
    CompanyCreated,
    #[serde(rename = "company.plan_ready")]
    CompanyPlanReady,
    #[serde(rename = "company.approved")]
    CompanyApproved,
    #[serde(rename = "company.deployed")]
    CompanyDeployed,

    // Budget
    #[serde(rename = "budget.warning")]
    BudgetWarning,
    #[serde(rename = "budget.exceeded")]
    BudgetExceeded,

    // Approval
    #[serde(rename = "approval.requested")]
    ApprovalRequested,
    #[serde(rename = "approval.granted")]
    ApprovalGranted,
    #[serde(rename = "approval.denied")]
    ApprovalDenied,

    // Message
    #[serde(rename = "message.sent")]
    MessageSent,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AjenEvent {
    pub id: String,
    pub company_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
