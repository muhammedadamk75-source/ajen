use ajen_core::traits::EventBus;
use ajen_core::types::event::AjenEvent;
use tokio::sync::broadcast;
use tracing::debug;

pub struct InMemoryEventBus {
    sender: broadcast::Sender<AjenEvent>,
}

impl InMemoryEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl EventBus for InMemoryEventBus {
    fn emit(&self, event: AjenEvent) {
        debug!(event_type = %event.event_type, company_id = %event.company_id, "event emitted");
        let _ = self.sender.send(event);
    }

    fn subscribe(&self) -> broadcast::Receiver<AjenEvent> {
        self.sender.subscribe()
    }
}
