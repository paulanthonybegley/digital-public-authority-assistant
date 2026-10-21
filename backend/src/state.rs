//! Shared application state.

use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    /// `None` in unit/integration tests that do not run Postgres.
    pub db: Option<PgPool>,
    /// `None` when Redis is not configured; cache and rate limiting fail open.
    pub redis: Option<ConnectionManager>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        let db = match &config.database_url {
            Some(url) => {
                let pool = PgPoolOptions::new()
                    .max_connections(10)
                    .acquire_timeout(std::time::Duration::from_secs(5))
                    .connect(url)
                    .await
                    .map_err(|e| {
                        AppError::DependencyUnavailable(format!("postgres connect failed: {e}"))
                    })?;
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| AppError::internal(anyhow::anyhow!("migrations failed: {e}")))?;
                tracing::info!("postgres connected, migrations applied");
                Some(pool)
            }
            None => {
                tracing::warn!("DATABASE_URL not set - audit persistence disabled");
                None
            }
        };

        let redis = match &config.redis_url {
            Some(url) => {
                let client = redis::Client::open(url.as_str())
                    .map_err(|e| AppError::DependencyUnavailable(format!("redis config: {e}")))?;
                let manager = ConnectionManager::new(client)
                    .await
                    .map_err(|e| AppError::DependencyUnavailable(format!("redis connect: {e}")))?;
                tracing::info!("redis connected");
                Some(manager)
            }
            None => {
                tracing::warn!("REDIS_URL not set - caching and rate limiting disabled");
                None
            }
        };

        Ok(Self { config, db, redis })
    }

    /// Build state without backing services (tests).
    pub fn for_tests(config: Config) -> Self {
        Self {
            config,
            db: None,
            redis: None,
        }
    }
}
