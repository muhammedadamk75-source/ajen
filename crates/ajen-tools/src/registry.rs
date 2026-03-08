use std::collections::HashMap;
use std::sync::Arc;

use ajen_core::traits::Tool;
use ajen_core::types::employee::EmployeeConfig;
use ajen_core::types::provider::ToolDefinition;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.spec().id.clone(), tool);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(id)
    }

    /// Get tools available for a specific employee based on their config
    pub fn get_for_employee(&self, config: &EmployeeConfig) -> Vec<Arc<dyn Tool>> {
        self.tools
            .values()
            .filter(|tool| {
                let spec = tool.spec();

                // Check tier
                if !spec.allowed_tiers.contains(&config.tier) {
                    return false;
                }

                // Check role restriction
                if let Some(ref allowed_roles) = spec.allowed_roles {
                    if !allowed_roles.contains(&config.role) {
                        return false;
                    }
                }

                // Check tool whitelist
                if !config.tools.is_empty() && !config.tools.contains(&spec.id) {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Convert tools to LLM-compatible tool definitions
    pub fn to_tool_definitions(&self, tools: &[Arc<dyn Tool>]) -> Vec<ToolDefinition> {
        tools
            .iter()
            .map(|tool| {
                let spec = tool.spec();
                ToolDefinition {
                    name: spec.name.clone(),
                    description: spec.description.clone(),
                    parameters: spec.parameters.clone(),
                }
            })
            .collect()
    }

    pub fn list(&self) -> Vec<&Arc<dyn Tool>> {
        self.tools.values().collect()
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
