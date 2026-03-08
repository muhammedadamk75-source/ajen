use crate::types::event::AjenEvent;

pub trait EventBus: Send + Sync {
    fn emit(&self, event: AjenEvent);
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AjenEvent>;
}
