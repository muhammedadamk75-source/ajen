use crate::types::director::CompanyRecord;

#[async_trait::async_trait]
pub trait CompanyStore: Send + Sync {
    async fn insert(&self, record: CompanyRecord) -> anyhow::Result<()>;
    async fn get(&self, company_id: &str) -> anyhow::Result<Option<CompanyRecord>>;
    async fn update(&self, record: CompanyRecord) -> anyhow::Result<()>;
    async fn list(&self) -> anyhow::Result<Vec<CompanyRecord>>;
}
