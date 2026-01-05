//! Storage backend benchmarks for unimorph-rs.
//!
//! This crate provides prototype implementations of storage backends
//! (SQLite, DuckDB, Parquet+Polars) to benchmark their performance
//! for UniMorph data access patterns.

pub mod duckdb;
pub mod error;
pub mod parquet;
pub mod sqlite;
pub mod types;

pub use error::Error;
pub use types::{DatasetStats, Entry, FeatureBundle, LangCode};

use std::collections::HashMap;

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Trait defining the storage interface for UniMorph data.
///
/// All storage backends implement this trait, allowing us to benchmark
/// them against the same workloads and potentially swap implementations.
pub trait Store {
    /// Initialize the store with data for a language.
    ///
    /// This may create tables, indexes, or write files depending on the backend.
    fn init(&mut self, lang: &LangCode, entries: &[Entry]) -> Result<()>;

    /// Look up all forms for a given lemma.
    ///
    /// This is the primary use case: "show me all inflections of *parlare*".
    fn lookup_by_lemma(&self, lang: &LangCode, lemma: &str) -> Result<Vec<Entry>>;

    /// Reverse lookup: find all entries that produce a given surface form.
    ///
    /// Example: "sono" in Italian could be from "essere" (to be) or "suonare" (to sound).
    fn lookup_by_form(&self, lang: &LangCode, form: &str) -> Result<Vec<Entry>>;

    /// Search for entries matching a feature pattern.
    ///
    /// Pattern supports wildcards: "V;IND;*;SG" matches any singular indicative verb.
    fn search_features(&self, lang: &LangCode, pattern: &str) -> Result<Vec<Entry>>;

    /// Get aggregate statistics for a language dataset.
    fn stats(&self, lang: &LangCode) -> Result<DatasetStats>;

    /// Count entries matching a feature across all loaded languages.
    ///
    /// Useful for cross-linguistic analysis.
    fn cross_lang_feature_count(&self, feature: &str) -> Result<HashMap<LangCode, usize>>;

    /// List all languages currently loaded in the store.
    fn languages(&self) -> Result<Vec<LangCode>>;
}
