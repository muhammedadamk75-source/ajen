use std::collections::HashMap;
use std::sync::Arc;

use ajen_core::traits::LLMProvider;

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn LLMProvider>> {
        self.providers.get(id)
    }

    pub fn first(&self) -> Option<&Arc<dyn LLMProvider>> {
        self.providers.values().next()
    }

    pub fn list(&self) -> Vec<&str> {
        self.providers.keys().map(|k| k.as_str()).collect()
    }
}

impl Clone for ProviderRegistry {
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
