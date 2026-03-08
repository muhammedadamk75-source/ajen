use std::sync::Arc;

use ajen_engine::context::EngineContext;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<EngineContext>,
    pub secret: String,
}
