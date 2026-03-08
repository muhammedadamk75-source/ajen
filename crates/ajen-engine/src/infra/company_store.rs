use std::collections::HashMap;

use ajen_core::traits::CompanyStore;
use ajen_core::types::director::CompanyRecord;
use tokio::sync::RwLock;

pub struct InMemoryCompanyStore {
    companies: RwLock<HashMap<String, CompanyRecord>>,
}

impl InMemoryCompanyStore {
    pub fn new() -> Self {
        Self {
            companies: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCompanyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CompanyStore for InMemoryCompanyStore {
    async fn insert(&self, record: CompanyRecord) -> anyhow::Result<()> {
        self.companies
            .write()
            .await
            .insert(record.id.clone(), record);
        Ok(())
    }

    async fn get(&self, company_id: &str) -> anyhow::Result<Option<CompanyRecord>> {
        Ok(self.companies.read().await.get(company_id).cloned())
    }

    async fn update(&self, record: CompanyRecord) -> anyhow::Result<()> {
        self.companies
            .write()
            .await
            .insert(record.id.clone(), record);
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<CompanyRecord>> {
        Ok(self.companies.read().await.values().cloned().collect())
    }
}
