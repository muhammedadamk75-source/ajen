use ajen_core::traits::LLMProvider;
use ajen_core::types::provider::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

// --- OpenAI API types ---

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionDef,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

// --- Conversion functions ---

fn convert_messages(messages: &[LLMMessage]) -> Vec<OpenAIMessage> {
    messages
        .iter()
        .map(|msg| match msg.role {
            MessageRole::System => OpenAIMessage {
                role: "system".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            MessageRole::User => OpenAIMessage {
                role: "user".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            MessageRole::Assistant => {
                let tool_calls = msg.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .map(|tc| OpenAIToolCall {
                            id: tc.id.clone(),
                            call_type: "function".to_string(),
                            function: OpenAIFunctionCall {
                                name: tc.name.clone(),
                                arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect()
                });
                OpenAIMessage {
                    role: "assistant".to_string(),
                    content: if msg.content.is_empty() {
                        None
                    } else {
                        Some(msg.content.clone())
                    },
                    tool_calls,
                    tool_call_id: None,
                }
            }
            MessageRole::Tool => OpenAIMessage {
                role: "tool".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: msg.tool_call_id.clone(),
            },
        })
        .collect()
}

fn convert_tools(tools: &[ToolDefinition]) -> Vec<OpenAITool> {
    tools
        .iter()
        .map(|t| OpenAITool {
            tool_type: "function".to_string(),
            function: OpenAIFunctionDef {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            },
        })
        .collect()
}

fn extract_response(response: OpenAIResponse) -> LLMResponse {
    let choice = &response.choices[0];
    let content = choice.message.content.clone().unwrap_or_default();

    let tool_calls = choice
        .message
        .tool_calls
        .as_ref()
        .map(|tcs| {
            tcs.iter()
                .map(|tc| ToolCallRequest {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                })
                .collect()
        })
        .unwrap_or_default();

    let finish_reason = match choice.finish_reason.as_deref() {
        Some("stop") => FinishReason::Stop,
        Some("tool_calls") => FinishReason::ToolUse,
        Some("length") => FinishReason::MaxTokens,
        _ => FinishReason::Unknown,
    };

    LLMResponse {
        content,
        tool_calls,
        usage: TokenUsage {
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
        },
        finish_reason,
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    fn id(&self) -> &str {
        "openai"
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn chat(&self, request: LLMRequest) -> Result<LLMResponse> {
        let messages = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let body = OpenAIRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools,
        };

        let response = self
            .client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {}: {}", status, body);
        }

        let api_response: OpenAIResponse = response.json().await?;
        Ok(extract_response(api_response))
    }

    fn estimate_cost(&self, model: &str, usage: &TokenUsage) -> f64 {
        crate::cost_table::calculate_cost_cents(model, usage)
    }

    async fn list_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }

    fn supports_tools(&self, _model: &str) -> bool {
        true
    }
}
