use ajen_core::TokenUsage;

/// Cost per million tokens (input_cost_per_m, output_cost_per_m) in cents
fn get_model_pricing(model: &str) -> (f64, f64) {
    match model {
        // Anthropic
        "claude-opus-4-6" | "claude-opus-4-20250514" => (1500.0, 7500.0),
        "claude-sonnet-4-6" | "claude-sonnet-4-20250514" => (300.0, 1500.0),
        m if m.starts_with("claude-haiku-4") => (25.0, 125.0),
        m if m.starts_with("claude-3-5-sonnet") => (300.0, 1500.0),
        m if m.starts_with("claude-3-5-haiku") => (100.0, 500.0),
        // OpenAI
        m if m.starts_with("gpt-4o") => (250.0, 1000.0),
        m if m.starts_with("gpt-4-turbo") => (1000.0, 3000.0),
        m if m.starts_with("gpt-4") => (3000.0, 6000.0),
        m if m.starts_with("gpt-3.5") => (50.0, 150.0),
        // Google Gemini
        m if m.starts_with("gemini-2.5-pro") => (125.0, 1000.0),
        m if m.starts_with("gemini-2.5-flash") => (15.0, 60.0),
        m if m.starts_with("gemini-2.0") => (10.0, 40.0),
        m if m.starts_with("gemini-1.5-pro") => (125.0, 500.0),
        m if m.starts_with("gemini-1.5-flash") => (7.5, 30.0),
        // Ollama (local, free)
        m if m.starts_with("ollama/") => (0.0, 0.0),
        // Default
        _ => (100.0, 300.0),
    }
}

/// Calculate cost in cents for a given model and token usage
pub fn calculate_cost_cents(model: &str, usage: &TokenUsage) -> f64 {
    let (input_per_m, output_per_m) = get_model_pricing(model);
    let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * input_per_m;
    let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * output_per_m;
    input_cost + output_cost
}
