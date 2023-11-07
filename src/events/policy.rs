use anyhow::Result;

#[async_trait::async_trait]
pub trait EventsPolicy: Send + Sync {
    async fn handle_event(&self, raw_event: Vec<u8>) -> Result<()>;
}
