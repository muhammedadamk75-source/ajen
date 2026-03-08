use std::path::PathBuf;

use ajen_core::traits::Tool;
use ajen_core::types::employee::EmployeeTier;
use ajen_core::types::tool::{ToolContext, ToolResult, ToolSpec};

pub struct ReadFileTool;

impl ReadFileTool {
    fn validate_path(work_dir: &str, file_path: &str) -> anyhow::Result<PathBuf> {
        let work = PathBuf::from(work_dir).canonicalize()?;
        let target = work.join(file_path).canonicalize()?;
        if !target.starts_with(&work) {
            anyhow::bail!("Path escapes working directory");
        }
        Ok(target)
    }
}

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn spec(&self) -> &ToolSpec {
        static SPEC: std::sync::LazyLock<ToolSpec> = std::sync::LazyLock::new(|| ToolSpec {
            id: "filesystem.read_file".to_string(),
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            category: "filesystem".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path to the file"
                    }
                },
                "required": ["path"]
            }),
            allowed_tiers: vec![
                EmployeeTier::Executive,
                EmployeeTier::Manager,
                EmployeeTier::Worker,
            ],
            allowed_roles: None,
            requires_approval: None,
        });
        &SPEC
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let start = std::time::Instant::now();
        let path = input["path"].as_str().unwrap_or("");

        match Self::validate_path(&context.work_dir, path) {
            Ok(target) => match tokio::fs::read_to_string(&target).await {
                Ok(content) => Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({ "content": content, "path": path }),
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    output: serde_json::Value::Null,
                    error: Some(format!("Failed to read file: {}", e)),
                    duration_ms: start.elapsed().as_millis() as u64,
                }),
            },
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }
}
