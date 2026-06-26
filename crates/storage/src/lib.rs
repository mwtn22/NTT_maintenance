//! Persistence layer (sqlx + PostgreSQL/PostGIS).
//!
//! Uses runtime queries (not the compile-time `query!` macros) so the
//! workspace builds without a live database / `DATABASE_URL`.

pub mod models;
pub mod repo;

pub use models::{
    EventFilter, EventRecord, FreshnessRecord, PrefCount, SourceRecord, SummaryRecord,
};
pub use repo::Repo;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}
