use ajen_core::traits::LLMProvider;
use ajen_core::types::provider::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

// --- Gemini API types ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiToolConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiToolConfig {
    function_declarations: Vec<GeminiFunctionDecl>,
}

#[derive(Debug, Serialize)]
struct GeminiFunctionDecl {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsage {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
}

// --- Conversion functions ---

fn convert_messages(messages: &[LLMMessage]) -> (Option<GeminiContent>, Vec<GeminiContent>) {
    let mut system_instruction = None;
    let mut contents = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::System => {
                system_instruction = Some(GeminiContent {
                    role: None,
                    parts: vec![GeminiPart {
                        text: Some(msg.content.clone()),
                        function_call: None,
                        function_response: None,
                    }],
                });
            }
            MessageRole::User => {
                contents.push(GeminiContent {
                    role: Some("user".to_string()),
                    parts: vec![GeminiPart {
                        text: Some(msg.content.clone()),
                        function_call: None,
                        function_response: None,
                    }],
                });
            }
            MessageRole::Assistant => {
                let mut parts = Vec::new();
                if !msg.content.is_empty() {
                    parts.push(GeminiPart {
                        text: Some(msg.content.clone()),
                        function_call: None,
                        function_response: None,
                    });
                }
                if let Some(tool_calls) = &msg.tool_calls {
                    for tc in tool_calls {
                        parts.push(GeminiPart {
                            text: None,
                            function_call: Some(GeminiFunctionCall {
                                name: tc.name.clone(),
                                args: tc.arguments.clone(),
                            }),
                            function_response: None,
                        });
                    }
                }
                if parts.is_empty() {
                    parts.push(GeminiPart {
                        text: Some(String::new()),
                        function_call: None,
                        function_response: None,
                    });
                }
                contents.push(GeminiContent {
                    role: Some("model".to_string()),
                    parts,
                });
            }
            MessageRole::Tool => {
                // Tool responses need the function name from the tool_call_id
                let name = msg.tool_call_id.clone().unwrap_or_default();
                let response_value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::json!({ "result": msg.content }));
                contents.push(GeminiContent {
                    role: Some("user".to_string()),
                    parts: vec![GeminiPart {
                        text: None,
                        function_call: None,
                        function_response: Some(GeminiFunctionResponse {
                            name,
                            response: response_value,
                        }),
                    }],
                });
            }
        }
    }

    (system_instruction, contents)
}

fn convert_tools(tools: &[ToolDefinition]) -> Vec<GeminiToolConfig> {
    vec![GeminiToolConfig {
        function_declarations: tools
            .iter()
            .map(|t| GeminiFunctionDecl {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            })
            .collect(),
    }]
}

fn extract_response(response: GeminiResponse) -> LLMResponse {
    let candidate = response.candidates.as_ref().and_then(|c| c.first());

    let mut text = String::new();
    let mut tool_calls = Vec::new();

    if let Some(candidate) = candidate {
        if let Some(content) = &candidate.content {
            for (i, part) in content.parts.iter().enumerate() {
                if let Some(t) = &part.text {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(t);
                }
                if let Some(fc) = &part.function_call {
                    tool_calls.push(ToolCallRequest {
                        id: format!("call_{}", i),
                        name: fc.name.clone(),
                        arguments: fc.args.clone(),
                    });
                }
            }
        }
    }

    let finish_reason = if !tool_calls.is_empty() {
        FinishReason::ToolUse
    } else {
        match candidate.and_then(|c| c.finish_reason.as_deref()) {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::MaxTokens,
            _ => FinishReason::Stop,
        }
    };

    let usage = response.usage_metadata.as_ref();
    LLMResponse {
        content: text,
        tool_calls,
        usage: TokenUsage {
            input_tokens: usage.and_then(|u| u.prompt_token_count).unwrap_or(0),
            output_tokens: usage.and_then(|u| u.candidates_token_count).unwrap_or(0),
        },
        finish_reason,
    }
}

#[async_trait::async_trait]
impl LLMProvider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini"
    }

    fn name(&self) -> &str {
        "Google Gemini"
    }

    async fn chat(&self, request: LLMRequest) -> Result<LLMResponse> {
        let (system_instruction, contents) = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let generation_config = if request.temperature.is_some() || request.max_tokens.is_some() {
            Some(GeminiGenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            })
        } else {
            None
        };

        let body = GeminiRequest {
            contents,
            system_instruction,
            tools,
            generation_config,
        };

        let url = format!(
            "{}{}:generateContent?key={}",
            API_BASE, request.model, self.api_key
        );

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error {}: {}", status, body);
        }

        let api_response: GeminiResponse = response.json().await?;
        Ok(extract_response(api_response))
    }

    fn estimate_cost(&self, model: &str, usage: &TokenUsage) -> f64 {
        crate::cost_table::calculate_cost_cents(model, usage)
    }

    async fn list_models(&self) -> Vec<String> {
        vec![
            "gemini-2.5-pro".to_string(),
            "gemini-2.5-flash".to_string(),
            "gemini-2.0-flash".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ]
    }

    fn supports_tools(&self, _model: &str) -> bool {
        true
    }
}
