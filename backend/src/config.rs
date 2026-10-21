//! Application configuration from the environment.
//!
//! All settings have safe defaults for local development; production is
//! configured via environment variables (see docker-compose.yml).

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// e.g. 0.0.0.0:8080
    pub bind_addr: String,
    /// Postgres connection string; when unset, persistence is disabled (tests).
    pub database_url: Option<String>,
    /// Redis connection string; when unset, cache/rate limiting are disabled.
    pub redis_url: Option<String>,
    /// Requests per minute per client before 429.
    pub rate_limit_per_minute: u64,
    /// CORS allow-origin, `*` for development.
    pub cors_allow_origin: String,
    /// When true, `/ready` fails if a configured backing service is missing.
    pub strict_readiness: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".to_string(),
            database_url: None,
            redis_url: None,
            rate_limit_per_minute: 120,
            cors_allow_origin: "*".to_string(),
            strict_readiness: false,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| Self::default().bind_addr),
            database_url: std::env::var("DATABASE_URL")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            redis_url: std::env::var("REDIS_URL")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                .ok()
                .map(|v| v.parse().context("RATE_LIMIT_PER_MINUTE must be a number"))
                .transpose()?
                .unwrap_or_else(Self::default_rate),
            cors_allow_origin: std::env::var("CORS_ALLOW_ORIGIN")
                .unwrap_or_else(|_| Self::default().cors_allow_origin),
            strict_readiness: std::env::var("STRICT_READINESS")
                .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
                .unwrap_or(false),
        })
    }

    fn default_rate() -> u64 {
        Self::default().rate_limit_per_minute
    }
}
