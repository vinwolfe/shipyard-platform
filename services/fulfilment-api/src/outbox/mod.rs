//! Transactional outbox subsystem.
//!
//! Why this exists:
//! - Orders and other workflows can enqueue side-effect intents in the same DB transaction.
//! - A worker later delivers these intents reliably with retry/backoff.

pub mod delivery;
pub mod repo;
pub mod types;
pub mod worker;
