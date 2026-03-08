use std::collections::HashMap;
use std::sync::Arc;

use ajen_core::traits::{CommsBus, EventBus};
use ajen_core::types::event::{AjenEvent, EventType};
use ajen_core::types::message::Message;
use chrono::Utc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub struct InMemoryCommsBus {
    channels: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    event_bus: Arc<dyn EventBus>,
}

impl InMemoryCommsBus {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }
}

#[async_trait::async_trait]
impl CommsBus for InMemoryCommsBus {
    fn subscribe(&self, employee_id: &str) -> mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        // Use blocking approach since we can't await in sync fn
        let channels = self.channels.clone();
        let eid = employee_id.to_string();
        tokio::spawn(async move {
            channels.write().await.insert(eid, tx);
        });
        rx
    }

    fn unsubscribe(&self, employee_id: &str) {
        let channels = self.channels.clone();
        let eid = employee_id.to_string();
        tokio::spawn(async move {
            channels.write().await.remove(&eid);
        });
    }

    async fn send(&self, mut message: Message) -> Message {
        message.id = Uuid::new_v4().to_string();
        message.created_at = Utc::now();

        self.event_bus.emit(AjenEvent {
            id: Uuid::new_v4().to_string(),
            company_id: message.company_id.clone(),
            employee_id: message.from_id.clone(),
            event_type: EventType::MessageSent,
            data: Some(serde_json::json!({
                "toId": message.to_id,
                "type": message.message_type,
            })),
            created_at: Utc::now(),
        });

        let channels = self.channels.read().await;
        if let Some(to_id) = &message.to_id {
            if let Some(tx) = channels.get(to_id) {
                let _ = tx.send(message.clone());
            }
        } else {
            // Broadcast
            for tx in channels.values() {
                let _ = tx.send(message.clone());
            }
        }

        message
    }
}
