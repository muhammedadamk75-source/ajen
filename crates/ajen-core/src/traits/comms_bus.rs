use crate::types::message::Message;

#[async_trait::async_trait]
pub trait CommsBus: Send + Sync {
    fn subscribe(&self, employee_id: &str) -> tokio::sync::mpsc::UnboundedReceiver<Message>;
    fn unsubscribe(&self, employee_id: &str);
    async fn send(&self, message: Message) -> Message;
}
