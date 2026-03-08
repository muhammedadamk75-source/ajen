use std::sync::Arc;

use ajen_core::types::employee::{EmployeeConfig, EmployeeTier};
use ajen_provider::model_selector::select_model;

use super::roles::{RolePromptContext, get_role_prompt};
use super::runtime::{EmployeeRuntime, RuntimeDeps};
use uuid::Uuid;

pub struct CreateEmployeeOptions {
    pub company_id: String,
    pub name: String,
    pub title: String,
    pub role: String,
    pub tier: Option<EmployeeTier>,
    pub manager_id: Option<String>,
    pub personality: Option<String>,
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub manifest_id: Option<String>,
    pub system_prompt: Option<String>,
}

fn default_tier_for_role(role: &str) -> EmployeeTier {
    match role {
        "ceo" | "cto" | "cmo" | "coo" => EmployeeTier::Executive,
        _ => EmployeeTier::Worker,
    }
}

pub fn create_employee(options: CreateEmployeeOptions, deps: Arc<RuntimeDeps>) -> EmployeeRuntime {
    let tier = options
        .tier
        .unwrap_or_else(|| default_tier_for_role(&options.role));
    let provider_id = options.provider_override.as_deref().unwrap_or("anthropic");
    let model = options
        .model_override
        .clone()
        .unwrap_or_else(|| select_model(tier, provider_id).model_id);

    let system_prompt = options.system_prompt.unwrap_or_else(|| {
        get_role_prompt(
            &options.role,
            &RolePromptContext {
                name: options.name.clone(),
                title: options.title.clone(),
                company_id: options.company_id.clone(),
                personality: options.personality.clone(),
            },
        )
    });

    let tools: Vec<String> = deps
        .tool_registry
        .list()
        .iter()
        .map(|t| t.spec().id.clone())
        .collect();

    let config = EmployeeConfig {
        id: Uuid::new_v4().to_string(),
        company_id: options.company_id,
        name: options.name,
        title: options.title,
        role: options.role,
        tier,
        manager_id: options.manager_id,
        provider_id: provider_id.to_string(),
        model_id: model,
        system_prompt,
        personality: options.personality,
        tools,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        config: serde_json::json!({}),
        manifest_id: options.manifest_id,
        container_status: None,
        worker_id: None,
    };

    EmployeeRuntime::new(config, deps)
}
