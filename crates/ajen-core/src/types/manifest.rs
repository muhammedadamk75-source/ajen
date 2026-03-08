use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::employee::EmployeeTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

fn default_author() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestModelConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

fn default_provider() -> String {
    "anthropic".to_string()
}

fn default_model() -> String {
    "claude-haiku-4-5-20251001".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCapabilities {
    #[serde(default)]
    pub can_delegate_work: bool,
    #[serde(default)]
    pub can_execute_shell: bool,
    #[serde(default)]
    pub can_access_internet: bool,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: u32,
}

fn default_max_concurrent() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestContainerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ContainerResources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestToolsConfig {
    #[serde(default)]
    pub builtin: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<Vec<CustomTool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    pub path: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestBudgetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_cents_per_task: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_threshold_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSpec {
    pub role: String,
    pub tier: EmployeeTier,
    pub model: ManifestModelConfig,
    pub persona: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    pub tools: ManifestToolsConfig,
    pub capabilities: ManifestCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ManifestContainerConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports_to: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<ManifestBudgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ManifestMetadata,
    pub spec: ManifestSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedManifest {
    #[serde(flatten)]
    pub manifest: EmployeeManifest,
    pub persona_content: String,
    #[serde(default)]
    pub skill_contents: Vec<String>,
}
