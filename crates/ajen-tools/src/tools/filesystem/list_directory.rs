use std::path::PathBuf;

use ajen_core::traits::Tool;
use ajen_core::types::employee::EmployeeTier;
use ajen_core::types::tool::{ToolContext, ToolResult, ToolSpec};

pub struct ListDirectoryTool;

impl ListDirectoryTool {
    fn validate_path(work_dir: &str, dir_path: &str) -> anyhow::Result<PathBuf> {
        let work = PathBuf::from(work_dir).canonicalize()?;
        let target = if dir_path.is_empty() || dir_path == "." {
            work.clone()
        } else {
            work.join(dir_path).canonicalize()?
        };
        if !target.starts_with(&work) {
            anyhow::bail!("Path escapes working directory");
        }
        Ok(target)
    }
}

#[async_trait::async_trait]
impl Tool for ListDirectoryTool {
    fn spec(&self) -> &ToolSpec {
        static SPEC: std::sync::LazyLock<ToolSpec> = std::sync::LazyLock::new(|| ToolSpec {
            id: "filesystem.list_directory".to_string(),
            name: "list_directory".to_string(),
            description: "List the contents of a directory".to_string(),
            category: "filesystem".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path to the directory (default: current directory)"
                    }
                },
                "required": []
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
        let path = input["path"].as_str().unwrap_or(".");

        match Self::validate_path(&context.work_dir, path) {
            Ok(target) => {
                let mut entries = Vec::new();
                match tokio::fs::read_dir(&target).await {
                    Ok(mut dir) => {
                        while let Ok(Some(entry)) = dir.next_entry().await {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let metadata = entry.metadata().await.ok();
                            let file_type = if metadata.as_ref().is_some_and(|m| m.is_dir()) {
                                "directory"
                            } else {
                                "file"
                            };
                            let size = metadata.map(|m| m.len()).unwrap_or(0);
                            entries.push(serde_json::json!({
                                "name": name,
                                "type": file_type,
                                "size": size,
                            }));
                        }
                        Ok(ToolResult {
                            success: true,
                            output: serde_json::json!({ "entries": entries }),
                            error: None,
                            duration_ms: start.elapsed().as_millis() as u64,
                        })
                    }
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(format!("Failed to read directory: {}", e)),
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
