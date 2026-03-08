use ajen_core::EmployeeTier;

pub struct ModelSelection {
    pub provider_id: String,
    pub model_id: String,
}

/// Select the appropriate model based on employee tier and provider
pub fn select_model(tier: EmployeeTier, provider_id: &str) -> ModelSelection {
    let model_id = match provider_id {
        "anthropic" => match tier {
            EmployeeTier::Executive => "claude-sonnet-4-6",
            EmployeeTier::Manager => "claude-sonnet-4-6",
            EmployeeTier::Worker => "claude-haiku-4-5-20251001",
        },
        "openai" => match tier {
            EmployeeTier::Executive => "gpt-4o",
            EmployeeTier::Manager => "gpt-4o",
            EmployeeTier::Worker => "gpt-4o-mini",
        },
        "gemini" => match tier {
            EmployeeTier::Executive => "gemini-2.5-pro",
            EmployeeTier::Manager => "gemini-2.5-flash",
            EmployeeTier::Worker => "gemini-2.0-flash",
        },
        "ollama" => match tier {
            EmployeeTier::Executive | EmployeeTier::Manager | EmployeeTier::Worker => "llama3.1",
        },
        _ => "claude-haiku-4-5-20251001",
    };

    ModelSelection {
        provider_id: provider_id.to_string(),
        model_id: model_id.to_string(),
    }
}
