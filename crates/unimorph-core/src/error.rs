//! Error types for unimorph-core.

use std::path::PathBuf;

/// Errors that can occur when working with UniMorph data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid ISO 639-3 language code.
    #[error("invalid language code: {0} (must be 3 lowercase ASCII letters)")]
    InvalidLangCode(String),

    /// Invalid feature bundle string.
    #[error("invalid feature bundle: {0}")]
    InvalidFeatureBundle(String),

    /// Malformed entry in TSV data.
    #[error("malformed entry at line {line}: {reason}")]
    MalformedEntry { line: usize, reason: String },

    /// Language not found in the store.
    #[error("language not found: {0}")]
    LanguageNotFound(String),

    /// Dataset download failed.
    #[error("download failed: {0}")]
    DownloadFailed(String),

    /// GitHub API rate limit exceeded.
    #[error("GitHub API rate limit exceeded, try again later")]
    RateLimited,

    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Cache directory not found or not writable.
    #[error("cache directory error: {path}: {reason}")]
    CacheDir { path: PathBuf, reason: String },

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
