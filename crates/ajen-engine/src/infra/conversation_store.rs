use std::collections::HashMap;

use ajen_core::traits::ConversationStore;
use ajen_core::types::provider::LLMMessage;
use tokio::sync::RwLock;

pub struct InMemoryConversationStore {
    conversations: RwLock<HashMap<String, Vec<LLMMessage>>>,
}

impl InMemoryConversationStore {
    pub fn new() -> Self {
        Self {
            conversations: RwLock::new(HashMap::new()),
        }
    }

    fn key(employee_id: &str, task_id: &str) -> String {
        format!("{}:{}", employee_id, task_id)
    }
}

impl Default for InMemoryConversationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ConversationStore for InMemoryConversationStore {
    async fn load(&self, employee_id: &str, task_id: &str) -> anyhow::Result<Vec<LLMMessage>> {
        let key = Self::key(employee_id, task_id);
        let conversations = self.conversations.read().await;
        Ok(conversations.get(&key).cloned().unwrap_or_default())
    }

    async fn append(
        &self,
        employee_id: &str,
        task_id: &str,
        message: LLMMessage,
    ) -> anyhow::Result<()> {
        let key = Self::key(employee_id, task_id);
        self.conversations
            .write()
            .await
            .entry(key)
            .or_default()
            .push(message);
        Ok(())
    }

    async fn clear(&self, employee_id: &str, task_id: &str) -> anyhow::Result<()> {
        let key = Self::key(employee_id, task_id);
        self.conversations.write().await.remove(&key);
        Ok(())
    }
}
