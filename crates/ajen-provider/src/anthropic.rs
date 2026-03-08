use ajen_core::traits::LLMProvider;
use ajen_core::types::provider::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

// --- Anthropic API request/response types ---

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// --- Conversion functions ---

fn convert_messages(messages: &[LLMMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system_prompt = None;
    let mut result = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::System => {
                system_prompt = Some(msg.content.clone());
            }
            MessageRole::User => {
                result.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Text(msg.content.clone()),
                });
            }
            MessageRole::Assistant => {
                if let Some(tool_calls) = &msg.tool_calls {
                    let mut blocks = Vec::new();
                    if !msg.content.is_empty() {
                        blocks.push(ContentBlock::Text {
                            text: msg.content.clone(),
                        });
                    }
                    for tc in tool_calls {
                        blocks.push(ContentBlock::ToolUse {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            input: tc.arguments.clone(),
                        });
                    }
                    result.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: AnthropicContent::Blocks(blocks),
                    });
                } else {
                    result.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: AnthropicContent::Text(msg.content.clone()),
                    });
                }
            }
            MessageRole::Tool => {
                let tool_call_id = msg.tool_call_id.clone().unwrap_or_default();
                result.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Blocks(vec![ContentBlock::ToolResult {
                        tool_use_id: tool_call_id,
                        content: msg.content.clone(),
                    }]),
                });
            }
        }
    }

    (system_prompt, result)
}

fn convert_tools(tools: &[ToolDefinition]) -> Vec<AnthropicTool> {
    tools
        .iter()
        .map(|t| AnthropicTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.parameters.clone(),
        })
        .collect()
}

fn extract_response(response: AnthropicResponse) -> LLMResponse {
    let mut text = String::new();
    let mut tool_calls = Vec::new();

    for block in response.content {
        match block {
            ContentBlock::Text { text: t } => {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(&t);
            }
            ContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(ToolCallRequest {
                    id,
                    name,
                    arguments: input,
                });
            }
            ContentBlock::ToolResult { .. } => {}
        }
    }

    let finish_reason = match response.stop_reason.as_deref() {
        Some("end_turn") | Some("stop") => FinishReason::Stop,
        Some("tool_use") => FinishReason::ToolUse,
        Some("max_tokens") => FinishReason::MaxTokens,
        _ => FinishReason::Unknown,
    };

    LLMResponse {
        content: text,
        tool_calls,
        usage: TokenUsage {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        },
        finish_reason,
    }
}

#[async_trait::async_trait]
impl LLMProvider for AnthropicProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn chat(&self, request: LLMRequest) -> Result<LLMResponse> {
        let (system, messages) = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let body = AnthropicRequest {
            model: request.model,
            max_tokens: request.max_tokens.unwrap_or(4096),
            system,
            messages,
            tools,
        };

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, body);
        }

        let api_response: AnthropicResponse = response.json().await?;
        Ok(extract_response(api_response))
    }

    fn estimate_cost(&self, model: &str, usage: &TokenUsage) -> f64 {
        crate::cost_table::calculate_cost_cents(model, usage)
    }

    async fn list_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".to_string(),
            "claude-sonnet-4-6".to_string(),
            "claude-haiku-4-5-20251001".to_string(),
        ]
    }

    fn supports_tools(&self, _model: &str) -> bool {
        true
    }
}
