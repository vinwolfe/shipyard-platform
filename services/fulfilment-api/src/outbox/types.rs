use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;
use std::fmt;

pub type OutboxId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxStatus {
    Pending,
    Processing,
    Sent,
    Failed,
}

impl OutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboxStatus::Pending => "PENDING",
            OutboxStatus::Processing => "PROCESSING",
            OutboxStatus::Sent => "SENT",
            OutboxStatus::Failed => "FAILED",
        }
    }

    pub fn from_db(s: &str) -> Self {
        match s {
            "PROCESSING" => OutboxStatus::Processing,
            "SENT" => OutboxStatus::Sent,
            "FAILED" => OutboxStatus::Failed,
            _ => OutboxStatus::Pending,
        }
    }
}

impl fmt::Display for OutboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Payload written by producers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: OutboxId,
    pub event_type: String,
    pub payload: Value,
}

/// Row shape read by the worker.
#[derive(Debug, sqlx::FromRow)]
pub struct OutboxRow {
    pub id: OutboxId,
    pub event_type: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i32,
}
