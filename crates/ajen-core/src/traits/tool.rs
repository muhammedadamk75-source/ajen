use crate::types::tool::{ToolContext, ToolResult, ToolSpec};

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> &ToolSpec;
    async fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> anyhow::Result<ToolResult>;
}
