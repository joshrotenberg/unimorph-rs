//! Error types for the benchmark crate.

use crate::LangCode;

/// Errors that can occur in storage operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid ISO 639-3 language code.
    #[error("invalid language code: {0} (expected 3 lowercase ASCII letters)")]
    InvalidLangCode(String),

    /// Malformed TSV entry.
    #[error("malformed entry at line {line}: {reason}")]
    MalformedEntry { line: usize, reason: String },

    /// Invalid feature bundle format.
    #[error("invalid feature bundle: {0}")]
    InvalidFeatureBundle(String),

    /// Dataset not found for the specified language.
    #[error("dataset not found for language: {0}")]
    DatasetNotFound(LangCode),

    /// Download failed.
    #[error("download failed: {0}")]
    DownloadFailed(String),

    /// Rate limited by GitHub API.
    #[error("rate limited by GitHub API")]
    RateLimited,

    /// Network error.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLite error.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// DuckDB error.
    #[error("DuckDB error: {0}")]
    DuckDb(#[from] duckdb::Error),

    /// Polars error.
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),
}
