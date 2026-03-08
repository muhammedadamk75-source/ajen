use crate::types::provider::{LLMRequest, LLMResponse, TokenUsage};

#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    async fn chat(&self, request: LLMRequest) -> anyhow::Result<LLMResponse>;
    fn estimate_cost(&self, model: &str, usage: &TokenUsage) -> f64;
    async fn list_models(&self) -> Vec<String>;
    fn supports_tools(&self, model: &str) -> bool;
}
