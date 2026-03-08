use std::collections::HashMap;

use ajen_core::traits::BudgetTracker;
use ajen_core::types::budget::{BudgetCheck, UsageRecord, UsageSummary};
use tokio::sync::RwLock;

pub struct InMemoryBudgetTracker {
    records: RwLock<Vec<UsageRecord>>,
    limits: RwLock<HashMap<String, i64>>,
}

impl InMemoryBudgetTracker {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            limits: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set_limit(&self, company_id: &str, employee_id: Option<&str>, limit_cents: i64) {
        let key = match employee_id {
            Some(eid) => format!("{}:{}", company_id, eid),
            None => company_id.to_string(),
        };
        self.limits.write().await.insert(key, limit_cents);
    }
}

impl Default for InMemoryBudgetTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BudgetTracker for InMemoryBudgetTracker {
    async fn record_usage(&self, record: UsageRecord) -> anyhow::Result<()> {
        self.records.write().await.push(record);
        Ok(())
    }

    async fn check_budget(
        &self,
        company_id: &str,
        employee_id: Option<&str>,
    ) -> anyhow::Result<BudgetCheck> {
        let records = self.records.read().await;
        let limits = self.limits.read().await;

        let used_cents: i64 = records
            .iter()
            .filter(|r| {
                r.company_id == company_id && employee_id.map_or(true, |eid| r.employee_id == eid)
            })
            .map(|r| r.cost_cents)
            .sum();

        let key = match employee_id {
            Some(eid) => format!("{}:{}", company_id, eid),
            None => company_id.to_string(),
        };
        let limit_cents = limits.get(&key).or_else(|| limits.get(company_id)).copied();

        let (allowed, percent_used) = match limit_cents {
            Some(limit) => (
                used_cents < limit,
                Some(used_cents as f64 / limit as f64 * 100.0),
            ),
            None => (true, None),
        };

        Ok(BudgetCheck {
            allowed,
            used_cents,
            limit_cents,
            percent_used,
        })
    }

    async fn get_usage_summary(&self, company_id: &str) -> anyhow::Result<UsageSummary> {
        let records = self.records.read().await;
        let mut by_employee = HashMap::new();
        let mut by_model = HashMap::new();
        let mut total = 0i64;

        for r in records.iter().filter(|r| r.company_id == company_id) {
            total += r.cost_cents;
            *by_employee.entry(r.employee_id.clone()).or_insert(0i64) += r.cost_cents;
            *by_model.entry(r.model_id.clone()).or_insert(0i64) += r.cost_cents;
        }

        Ok(UsageSummary {
            total_cost_cents: total,
            by_employee,
            by_model,
        })
    }
}
