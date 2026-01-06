//! Core library for working with UniMorph morphological data.
//!
//! This crate provides types, storage, and query capabilities for UniMorph
//! datasets. It is designed for high-performance lookups (conjugation/declension
//! queries) while also supporting bulk export for ML pipelines.
//!
//! # Quick Start
//!
//! ```ignore
//! use unimorph_core::{Repository, Store};
//!
//! #[tokio::main]
//! async fn main() -> unimorph_core::Result<()> {
//!     // Initialize repository (handles downloads and caching)
//!     let repo = Repository::new()?;
//!
//!     // Ensure Italian dataset is available
//!     repo.ensure("ita").await?;
//!
//!     // Open the store for queries
//!     let store = repo.store()?;
//!
//!     // Look up all forms of "parlare"
//!     for entry in store.inflect("ita", "parlare")? {
//!         println!("{} -> {} ({})", entry.lemma, entry.form, entry.features);
//!     }
//!
//!     // Reverse lookup: what lemmas produce "parlo"?
//!     for entry in store.analyze("ita", "parlo")? {
//!         println!("{} <- {} ({})", entry.form, entry.lemma, entry.features);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! - **SQLite backend**: All data stored in a single file at `~/.cache/unimorph/datasets.db`
//! - **Pre-computed stats**: Aggregate statistics cached at import time
//! - **Iterator-based queries**: Results stream from SQLite, won't OOM on large datasets
//! - **Parquet export**: For users who need DataFrame integration

pub mod error;
pub mod export;
pub mod query;
pub mod repository;
pub mod store;
pub mod types;

pub use error::{Error, Result};
#[cfg(feature = "parquet")]
pub use export::ParquetExportOptions;
pub use query::QueryBuilder;
pub use repository::{DownloadProgress, Repository};
pub use store::Store;
pub use types::{DatasetStats, Entry, FeatureBundle, LangCode};
