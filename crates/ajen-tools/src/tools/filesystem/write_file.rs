use std::path::PathBuf;

use ajen_core::traits::Tool;
use ajen_core::types::employee::EmployeeTier;
use ajen_core::types::tool::{ToolContext, ToolResult, ToolSpec};

pub struct WriteFileTool;

impl WriteFileTool {
    fn validate_path(work_dir: &str, file_path: &str) -> anyhow::Result<PathBuf> {
        let work = PathBuf::from(work_dir).canonicalize()?;
        // For write, the file may not exist yet — canonicalize the parent
        let target = work.join(file_path);
        if let Some(parent) = target.parent() {
            // Ensure parent dir is within work_dir (create if needed)
            let parent_canonical = if parent.exists() {
                parent.canonicalize()?
            } else {
                // Parent doesn't exist, check that its ancestor is within work_dir
                let mut check = parent.to_path_buf();
                while !check.exists() {
                    check = check.parent().unwrap_or(&work).to_path_buf();
                }
                let canonical = check.canonicalize()?;
                if !canonical.starts_with(&work) {
                    anyhow::bail!("Path escapes working directory");
                }
                // Return the intended (non-canonical) path since parent doesn't exist yet
                parent.to_path_buf()
            };
            if parent_canonical.exists() && !parent_canonical.starts_with(&work) {
                anyhow::bail!("Path escapes working directory");
            }
        }
        Ok(target)
    }
}

#[async_trait::async_trait]
impl Tool for WriteFileTool {
    fn spec(&self) -> &ToolSpec {
        static SPEC: std::sync::LazyLock<ToolSpec> = std::sync::LazyLock::new(|| ToolSpec {
            id: "filesystem.write_file".to_string(),
            name: "write_file".to_string(),
            description: "Write content to a file, creating parent directories if needed"
                .to_string(),
            category: "filesystem".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
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
        let content = input["content"].as_str().unwrap_or("");

        match Self::validate_path(&context.work_dir, path) {
            Ok(target) => {
                if let Some(parent) = target.parent() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        return Ok(ToolResult {
                            success: false,
                            output: serde_json::Value::Null,
                            error: Some(format!("Failed to create directories: {}", e)),
                            duration_ms: start.elapsed().as_millis() as u64,
                        });
                    }
                }
                match tokio::fs::write(&target, content).await {
                    Ok(()) => Ok(ToolResult {
                        success: true,
                        output: serde_json::json!({
                            "path": path,
                            "bytesWritten": content.len()
                        }),
                        error: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(format!("Failed to write file: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }),
                }
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }
}
