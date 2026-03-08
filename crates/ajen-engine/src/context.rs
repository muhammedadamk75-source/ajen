use std::path::Path;
use std::sync::Arc;

use ajen_core::traits::{
    BudgetTracker, CommsBus, CompanyStore, ConversationStore, EventBus, MemoryStore,
};
use ajen_provider::anthropic::AnthropicProvider;
use ajen_provider::gemini::GeminiProvider;
use ajen_provider::ollama::OllamaProvider;
use ajen_provider::openai::OpenAIProvider;
use ajen_provider::registry::ProviderRegistry;
use ajen_tools::registry::ToolRegistry;
use ajen_tools::tools::filesystem;
use tracing::info;

use crate::infra::budget_tracker::InMemoryBudgetTracker;
use crate::infra::comms_bus::InMemoryCommsBus;
use crate::infra::company_store::InMemoryCompanyStore;
use crate::infra::conversation_store::InMemoryConversationStore;
use crate::infra::event_bus::InMemoryEventBus;
use crate::infra::memory_store::InMemoryMemoryStore;
use crate::manifest::registry::ManifestRegistry;

pub struct EngineConfig {
    pub database_url: Option<String>,
    pub workspace_dir: String,
    pub manifests_dir: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    pub ollama_enabled: Option<bool>,
    pub budget_company_limit_cents: Option<i64>,
    pub budget_per_employee_limit_cents: Option<i64>,
}

pub struct EngineContext {
    pub event_bus: Arc<dyn EventBus>,
    pub comms_bus: Arc<dyn CommsBus>,
    pub memory_store: Arc<dyn MemoryStore>,
    pub budget_tracker: Arc<dyn BudgetTracker>,
    pub conversation_store: Arc<dyn ConversationStore>,
    pub company_store: Arc<dyn CompanyStore>,
    pub tool_registry: ToolRegistry,
    pub manifest_registry: ManifestRegistry,
    pub provider_registry: ProviderRegistry,
    pub config: EngineConfig,
}

impl EngineContext {
    pub async fn create(config: EngineConfig) -> anyhow::Result<Self> {
        // Event bus
        let event_bus: Arc<dyn EventBus> = Arc::new(InMemoryEventBus::default());

        // Comms bus
        let comms_bus: Arc<dyn CommsBus> = Arc::new(InMemoryCommsBus::new(event_bus.clone()));

        // Memory store
        let memory_store: Arc<dyn MemoryStore> = Arc::new(InMemoryMemoryStore::default());

        // Budget tracker
        let budget_tracker: Arc<dyn BudgetTracker> = Arc::new(InMemoryBudgetTracker::default());

        // Conversation store
        let conversation_store: Arc<dyn ConversationStore> =
            Arc::new(InMemoryConversationStore::default());

        // Company store
        let company_store: Arc<dyn CompanyStore> = Arc::new(InMemoryCompanyStore::default());

        // Tool registry
        let mut tool_registry = ToolRegistry::new();
        for tool in filesystem::register_all() {
            tool_registry.register(tool);
        }
        info!(tools = tool_registry.list().len(), "tools registered");

        // Provider registry
        let mut provider_registry = ProviderRegistry::new();
        if let Some(ref api_key) = config.anthropic_api_key {
            provider_registry.register(Arc::new(AnthropicProvider::new(api_key.clone())));
            info!("Anthropic provider registered");
        }
        if let Some(ref api_key) = config.openai_api_key {
            provider_registry.register(Arc::new(OpenAIProvider::new(api_key.clone())));
            info!("OpenAI provider registered");
        }
        if let Some(ref api_key) = config.gemini_api_key {
            provider_registry.register(Arc::new(GeminiProvider::new(api_key.clone())));
            info!("Gemini provider registered");
        }
        if config.ollama_enabled.unwrap_or(false) {
            let base = config
                .ollama_base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".into());
            provider_registry.register(Arc::new(OllamaProvider::new(base)));
            info!("Ollama provider registered");
        }

        // Manifest registry
        let mut manifest_registry = ManifestRegistry::new();
        if let Some(ref dir) = config.manifests_dir {
            let path = Path::new(dir);
            if path.exists() {
                manifest_registry.load_from_directory(path).await?;
                info!(
                    manifests = manifest_registry.list().len(),
                    "manifests loaded"
                );
            }
        }

        Ok(Self {
            event_bus,
            comms_bus,
            memory_store,
            budget_tracker,
            conversation_store,
            company_store,
            tool_registry,
            manifest_registry,
            provider_registry,
            config,
        })
    }
}
