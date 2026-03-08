use ajen_core::traits::LLMProvider;
use ajen_core::types::provider::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

// --- Ollama uses OpenAI-compatible API types ---

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatTool>>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: ChatFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct ChatTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ChatFunctionDef,
}

#[derive(Debug, Serialize)]
struct ChatFunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: ChatUsage,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ChatUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModel>>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

// --- Conversion functions ---

fn convert_messages(messages: &[LLMMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .map(|msg| match msg.role {
            MessageRole::System => ChatMessage {
                role: "system".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            MessageRole::User => ChatMessage {
                role: "user".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            MessageRole::Assistant => {
                let tool_calls = msg.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .map(|tc| ChatToolCall {
                            id: tc.id.clone(),
                            call_type: "function".to_string(),
                            function: ChatFunctionCall {
                                name: tc.name.clone(),
                                arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect()
                });
                ChatMessage {
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
            MessageRole::Tool => ChatMessage {
                role: "tool".to_string(),
                content: Some(msg.content.clone()),
                tool_calls: None,
                tool_call_id: msg.tool_call_id.clone(),
            },
        })
        .collect()
}

fn convert_tools(tools: &[ToolDefinition]) -> Vec<ChatTool> {
    tools
        .iter()
        .map(|t| ChatTool {
            tool_type: "function".to_string(),
            function: ChatFunctionDef {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            },
        })
        .collect()
}

fn extract_response(response: ChatResponse) -> LLMResponse {
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
impl LLMProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    async fn chat(&self, request: LLMRequest) -> Result<LLMResponse> {
        let messages = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let body = ChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools,
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error {}: {}", status, body);
        }

        let api_response: ChatResponse = response.json().await?;
        Ok(extract_response(api_response))
    }

    fn estimate_cost(&self, _model: &str, _usage: &TokenUsage) -> f64 {
        0.0
    }

    async fn list_models(&self) -> Vec<String> {
        let url = format!("{}/api/tags", self.base_url);
        let Ok(response) = self.client.get(&url).send().await else {
            return vec![];
        };
        let Ok(tags) = response.json::<OllamaTagsResponse>().await else {
            return vec![];
        };
        tags.models
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect()
    }

    fn supports_tools(&self, _model: &str) -> bool {
        true
    }
}
