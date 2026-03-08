use std::sync::Arc;

use ajen_core::traits::{BudgetTracker, CommsBus, ConversationStore, EventBus, MemoryStore};
use ajen_core::types::budget::UsageRecord;
use ajen_core::types::employee::{EmployeeConfig, EmployeeState, EmployeeStatus};
use ajen_core::types::event::{AjenEvent, EventType};
use ajen_core::types::provider::{FinishReason, LLMMessage, LLMRequest};
use ajen_provider::cost_table::calculate_cost_cents;
use ajen_provider::registry::ProviderRegistry;
use ajen_tools::registry::ToolRegistry;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const MAX_ITERATIONS: u32 = 50;

pub struct RuntimeDeps {
    pub event_bus: Arc<dyn EventBus>,
    pub comms_bus: Arc<dyn CommsBus>,
    pub memory_store: Arc<dyn MemoryStore>,
    pub budget_tracker: Arc<dyn BudgetTracker>,
    pub conversation_store: Arc<dyn ConversationStore>,
    pub tool_registry: ToolRegistry,
    pub provider_registry: ProviderRegistry,
    pub work_dir: String,
}

pub struct EmployeeRuntime {
    pub config: EmployeeConfig,
    state: RwLock<EmployeeState>,
    conversation: RwLock<Vec<LLMMessage>>,
    deps: Arc<RuntimeDeps>,
}

impl EmployeeRuntime {
    pub fn new(config: EmployeeConfig, deps: Arc<RuntimeDeps>) -> Self {
        Self {
            state: RwLock::new(EmployeeState {
                status: EmployeeStatus::Idle,
                current_task_id: None,
                last_active_at: Utc::now(),
            }),
            conversation: RwLock::new(Vec::new()),
            config,
            deps,
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        // Load memories
        let memories = self
            .deps
            .memory_store
            .recall(ajen_core::traits::RecallQuery {
                company_id: self.config.company_id.clone(),
                employee_id: Some(self.config.id.clone()),
                types: None,
                limit: Some(50),
                include_shared: true,
                search: None,
            })
            .await?;

        // Build system prompt with memories
        let mut system = self.config.system_prompt.clone();
        if !memories.is_empty() {
            system.push_str("\n\nRELEVANT MEMORIES:\n");
            for mem in &memories {
                system.push_str(&format!("- [{:?}] {}\n", mem.memory_type, mem.content));
            }
        }

        self.conversation
            .write()
            .await
            .push(LLMMessage::system(system));

        // Emit spawned event
        self.emit(
            EventType::EmployeeSpawned,
            Some(serde_json::json!({
                "name": self.config.name,
                "role": self.config.role,
                "tier": self.config.tier,
            })),
        );

        info!(
            employee = %self.config.name,
            role = %self.config.role,
            "employee initialized"
        );

        Ok(())
    }

    pub async fn execute_task(&self, task_id: &str, instruction: &str) -> anyhow::Result<String> {
        // Update state
        {
            let mut state = self.state.write().await;
            state.status = EmployeeStatus::Working;
            state.current_task_id = Some(task_id.to_string());
            state.last_active_at = Utc::now();
        }

        self.emit(
            EventType::TaskStarted,
            Some(serde_json::json!({
                "taskId": task_id,
            })),
        );

        // Add user instruction
        self.conversation
            .write()
            .await
            .push(LLMMessage::user(instruction));

        let mut iterations = 0u32;
        let mut final_response = String::new();

        loop {
            if iterations >= MAX_ITERATIONS {
                warn!(employee = %self.config.name, "max iterations reached");
                break;
            }
            iterations += 1;

            // Budget check
            let budget = self
                .deps
                .budget_tracker
                .check_budget(&self.config.company_id, Some(&self.config.id))
                .await?;
            if !budget.allowed {
                self.emit(
                    EventType::BudgetExceeded,
                    Some(serde_json::json!({
                        "usedCents": budget.used_cents,
                        "limitCents": budget.limit_cents,
                    })),
                );
                final_response = "[Budget exceeded — stopping]".to_string();
                break;
            }

            // Get tools for this employee
            let tools = self.deps.tool_registry.get_for_employee(&self.config);
            let tool_defs = self.deps.tool_registry.to_tool_definitions(&tools);

            // Build LLM request
            let conversation = self.conversation.read().await.clone();
            let request = LLMRequest {
                model: self.config.model_id.clone(),
                messages: conversation,
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens.or(Some(4096)),
                tools: if tool_defs.is_empty() {
                    None
                } else {
                    Some(tool_defs)
                },
                stream: None,
            };

            self.emit(
                EventType::LlmRequest,
                Some(serde_json::json!({
                    "model": self.config.model_id,
                    "messageCount": request.messages.len(),
                })),
            );

            // Call LLM — use the provider specified in employee config
            let provider = self
                .deps
                .provider_registry
                .get(&self.config.provider_id)
                .or_else(|| self.deps.provider_registry.first())
                .ok_or_else(|| {
                    anyhow::anyhow!("No provider '{}' registered", self.config.provider_id)
                })?;

            let response = provider.chat(request).await?;

            // Record usage
            let cost = calculate_cost_cents(&self.config.model_id, &response.usage);
            self.deps
                .budget_tracker
                .record_usage(UsageRecord {
                    company_id: self.config.company_id.clone(),
                    employee_id: self.config.id.clone(),
                    task_id: Some(task_id.to_string()),
                    provider_id: self.config.provider_id.clone(),
                    model_id: self.config.model_id.clone(),
                    tokens_in: response.usage.input_tokens,
                    tokens_out: response.usage.output_tokens,
                    cost_cents: cost as i64,
                })
                .await?;

            self.emit(
                EventType::LlmResponse,
                Some(serde_json::json!({
                    "tokensIn": response.usage.input_tokens,
                    "tokensOut": response.usage.output_tokens,
                    "costCents": cost as i64,
                    "toolCalls": response.tool_calls.len(),
                })),
            );

            // Check if done
            if response.finish_reason == FinishReason::Stop || response.tool_calls.is_empty() {
                final_response = response.content.clone();
                self.conversation
                    .write()
                    .await
                    .push(LLMMessage::assistant(&response.content));
                break;
            }

            // Add assistant message with tool calls
            self.conversation
                .write()
                .await
                .push(LLMMessage::assistant_with_tools(
                    &response.content,
                    response.tool_calls.clone(),
                ));

            // Execute each tool call
            for tool_call in &response.tool_calls {
                debug!(tool = %tool_call.name, "executing tool");

                self.emit(
                    EventType::ToolCalled,
                    Some(serde_json::json!({
                        "toolId": tool_call.name,
                        "taskId": task_id,
                    })),
                );

                let context = ajen_core::ToolContext {
                    company_id: self.config.company_id.clone(),
                    employee_id: self.config.id.clone(),
                    task_id: Some(task_id.to_string()),
                    work_dir: self.deps.work_dir.clone(),
                };

                let result = if let Some(tool) = self.deps.tool_registry.get(&tool_call.name) {
                    tool.execute(tool_call.arguments.clone(), &context).await
                } else {
                    Ok(ajen_core::ToolResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(format!("Tool '{}' not found", tool_call.name)),
                        duration_ms: 0,
                    })
                };

                match result {
                    Ok(ref tool_result) => {
                        let event_type = if tool_result.success {
                            EventType::ToolCompleted
                        } else {
                            EventType::ToolFailed
                        };
                        self.emit(
                            event_type,
                            Some(serde_json::json!({
                                "toolId": tool_call.name,
                                "durationMs": tool_result.duration_ms,
                                "success": tool_result.success,
                            })),
                        );

                        let output = serde_json::to_string(&tool_result.output)
                            .unwrap_or_else(|_| "null".to_string());
                        self.conversation
                            .write()
                            .await
                            .push(LLMMessage::tool(&tool_call.id, &output));
                    }
                    Err(e) => {
                        error!(tool = %tool_call.name, error = %e, "tool execution error");
                        self.emit(
                            EventType::ToolFailed,
                            Some(serde_json::json!({
                                "toolId": tool_call.name,
                                "error": e.to_string(),
                            })),
                        );
                        self.conversation
                            .write()
                            .await
                            .push(LLMMessage::tool(&tool_call.id, &format!("Error: {}", e)));
                    }
                }
            }
        }

        // Reset state
        {
            let mut state = self.state.write().await;
            state.status = EmployeeStatus::Idle;
            state.current_task_id = None;
            state.last_active_at = Utc::now();
        }

        self.emit(
            EventType::TaskCompleted,
            Some(serde_json::json!({
                "taskId": task_id,
                "iterations": iterations,
            })),
        );

        info!(
            employee = %self.config.name,
            task_id = %task_id,
            iterations = iterations,
            "task completed"
        );

        Ok(final_response)
    }

    pub async fn terminate(&self) {
        {
            let mut state = self.state.write().await;
            state.status = EmployeeStatus::Terminated;
        }
        self.deps.comms_bus.unsubscribe(&self.config.id);
        self.emit(EventType::EmployeeTerminated, None);
    }

    pub async fn state(&self) -> EmployeeState {
        self.state.read().await.clone()
    }

    fn emit(&self, event_type: EventType, data: Option<serde_json::Value>) {
        self.deps.event_bus.emit(AjenEvent {
            id: Uuid::new_v4().to_string(),
            company_id: self.config.company_id.clone(),
            employee_id: Some(self.config.id.clone()),
            event_type,
            data,
            created_at: Utc::now(),
        });
    }
}
