use crate::types::memory::{Memory, MemoryType};

#[derive(Debug, Clone)]
pub struct RecallQuery {
    pub company_id: String,
    pub employee_id: Option<String>,
    pub types: Option<Vec<MemoryType>>,
    pub limit: Option<u32>,
    pub include_shared: bool,
    pub search: Option<String>,
}

#[async_trait::async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&self, memory: Memory) -> anyhow::Result<Memory>;
    async fn recall(&self, query: RecallQuery) -> anyhow::Result<Vec<Memory>>;
    async fn forget(&self, memory_id: &str) -> anyhow::Result<()>;
}
