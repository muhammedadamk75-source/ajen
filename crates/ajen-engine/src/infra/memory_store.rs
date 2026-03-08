use ajen_core::traits::{MemoryStore, RecallQuery};
use ajen_core::types::memory::Memory;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct InMemoryMemoryStore {
    memories: RwLock<Vec<Memory>>,
}

impl InMemoryMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoryStore for InMemoryMemoryStore {
    async fn store(&self, mut memory: Memory) -> anyhow::Result<Memory> {
        memory.id = Uuid::new_v4().to_string();
        memory.created_at = Utc::now();
        self.memories.write().await.push(memory.clone());
        Ok(memory)
    }

    async fn recall(&self, query: RecallQuery) -> anyhow::Result<Vec<Memory>> {
        let memories = self.memories.read().await;
        let limit = query.limit.unwrap_or(50) as usize;

        let mut results: Vec<&Memory> = memories
            .iter()
            .filter(|m| {
                if m.company_id != query.company_id {
                    return false;
                }
                if let Some(ref eid) = query.employee_id {
                    if query.include_shared {
                        if m.employee_id.as_ref() != Some(eid) && m.employee_id.is_some() {
                            return false;
                        }
                    } else if m.employee_id.as_ref() != Some(eid) {
                        return false;
                    }
                }
                if let Some(ref types) = query.types {
                    if !types.contains(&m.memory_type) {
                        return false;
                    }
                }
                if let Some(ref search) = query.search {
                    if !m.content.to_lowercase().contains(&search.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .collect();

        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit);

        Ok(results.into_iter().cloned().collect())
    }

    async fn forget(&self, memory_id: &str) -> anyhow::Result<()> {
        self.memories.write().await.retain(|m| m.id != memory_id);
        Ok(())
    }
}
