use crate::types::provider::LLMMessage;

#[async_trait::async_trait]
pub trait ConversationStore: Send + Sync {
    async fn load(&self, employee_id: &str, task_id: &str) -> anyhow::Result<Vec<LLMMessage>>;
    async fn append(
        &self,
        employee_id: &str,
        task_id: &str,
        message: LLMMessage,
    ) -> anyhow::Result<()>;
    async fn clear(&self, employee_id: &str, task_id: &str) -> anyhow::Result<()>;
}
