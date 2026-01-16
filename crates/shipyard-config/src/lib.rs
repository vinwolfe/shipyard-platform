//! shipyard-config
//!
//! Why this crate exists:
//! - Defines a stable runtime configuration contract for services.
//! - Provides one golden-path loader (env → typed struct).
//! - Fails fast at startup when config is invalid.
//!
//! NOTE: Add additional config fields only when concrete consumers require them.

use serde::Deserialize;
use thiserror::Error;

const DEFAULT_SERVICE_PORT: u16 = 8080;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Dev,
    Test,
    Prod,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Runtime environment (dev/test/prod)
    #[serde(default)]
    pub env: Environment,

    /// HTTP port the service listens on
    #[serde(default = "default_service_port")]
    pub service_port: u16,

    /// OTLP endpoint for traces/metrics export (wired later by observability work)
    #[serde(default)]
    pub otel_exporter_otlp_endpoint: Option<String>,
}

fn default_service_port() -> u16 {
    DEFAULT_SERVICE_PORT
}

impl AppConfig {
    /// Load config from process environment variables (fail fast)
    pub fn from_env() -> Result<Self, ConfigError> {
        let cfg: AppConfig = envy::from_env().map_err(ConfigError::Parse)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Load config from an iterator of key/value pairs (useful for tests)
    pub fn from_kv<I, K, V>(iter: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let owned: Vec<(String, String)> = iter
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect();

        let cfg: AppConfig = envy::from_iter(owned).map_err(ConfigError::Parse)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Dev-like config that cannot drift from defaults + validation.
    pub fn dev() -> Self {
        Self::from_kv(std::iter::empty::<(&str, &str)>())
            .expect("default config should always be valid")
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.service_port == 0 {
            return Err(ConfigError::Validation(
                "service_port must be in 1..=65535 (env: SERVICE_PORT)".to_string(),
            ));
        }

        if let Some(ep) = &self.otel_exporter_otlp_endpoint
            && ep.trim().is_empty()
        {
            return Err(ConfigError::Validation(
                "otel_exporter_otlp_endpoint must not be empty when set (env: OTEL_EXPORTER_OTLP_ENDPOINT)".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse configuration from environment: {0}")]
    Parse(envy::Error),

    #[error("invalid configuration: {0}")]
    Validation(String),
}
