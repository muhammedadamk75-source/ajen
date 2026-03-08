use crate::types::budget::{BudgetCheck, UsageRecord, UsageSummary};

#[async_trait::async_trait]
pub trait BudgetTracker: Send + Sync {
    async fn record_usage(&self, record: UsageRecord) -> anyhow::Result<()>;
    async fn check_budget(
        &self,
        company_id: &str,
        employee_id: Option<&str>,
    ) -> anyhow::Result<BudgetCheck>;
    async fn get_usage_summary(&self, company_id: &str) -> anyhow::Result<UsageSummary>;
}
