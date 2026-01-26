use async_trait::async_trait;

#[derive(Debug)]
pub struct DeliveryError {
    pub message: String,
}

impl DeliveryError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

pub type DeliveryResult = Result<(), DeliveryError>;

/// Delivery abstraction (seam).
///
/// Today: just log to prove the pipeline.
/// Soon: publish event, call external API, or route to DLQ.
#[async_trait]
pub trait DeliverySink: Send + Sync {
    async fn deliver(&self, event_type: &str, payload: &serde_json::Value) -> DeliveryResult;
}

/// Minimal sink: log “delivered”.
pub struct LogSink;

#[async_trait]
impl DeliverySink for LogSink {
    async fn deliver(&self, event_type: &str, payload: &serde_json::Value) -> DeliveryResult {
        tracing::info!(
            event_type = %event_type,
            payload = %payload,
            message = "outbox.delivered"
        );
        Ok(())
    }
}
