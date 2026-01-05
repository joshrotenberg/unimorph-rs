//! Repository for downloading and caching UniMorph datasets.
//!
//! The repository manages the local cache of UniMorph data, handling downloads
//! from GitHub and import into the SQLite store.
//!
//! # Cache Location
//!
//! By default, data is stored in:
//! - Linux: `~/.cache/unimorph/`
//! - macOS: `~/Library/Caches/unimorph/`
//! - Windows: `%LOCALAPPDATA%\unimorph\`
//!
//! # Example
//!
//! ```ignore
//! use unimorph_core::Repository;
//!
//! let repo = Repository::new()?;
//!
//! // Download and import Italian
//! repo.ensure("ita").await?;
//!
//! // Query the data
//! let store = repo.store()?;
//! for entry in store.inflect("ita", "parlare")? {
//!     println!("{}", entry.form);
//! }
//! ```

use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument, warn};

use crate::{Entry, Error, LangCode, Result, Store};

const UNIMORPH_RAW_URL: &str = "https://raw.githubusercontent.com/unimorph";

/// Repository for managing UniMorph datasets.
///
/// Handles downloading from GitHub and importing into the local SQLite store.
pub struct Repository {
    cache_dir: PathBuf,
    store: Store,
}

impl Repository {
    /// Create a new repository using the default cache directory.
    ///
    /// The default location is platform-specific:
    /// - Linux: `~/.cache/unimorph/`
    /// - macOS: `~/Library/Caches/unimorph/`
    /// - Windows: `%LOCALAPPDATA%\unimorph\`
    #[instrument(level = "debug")]
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| Error::CacheDir {
                path: PathBuf::from("~/.cache"),
                reason: "could not determine cache directory".to_string(),
            })?
            .join("unimorph");

        debug!(cache_dir = %cache_dir.display(), "using default cache directory");
        Self::with_cache_dir(cache_dir)
    }

    /// Create a repository with a custom cache directory.
    pub fn with_cache_dir<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir).map_err(|e| Error::CacheDir {
            path: cache_dir.clone(),
            reason: e.to_string(),
        })?;

        let db_path = cache_dir.join("datasets.db");
        let store = Store::open(&db_path)?;

        Ok(Self { cache_dir, store })
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get a reference to the underlying store.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Get a mutable reference to the underlying store.
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }

    /// Ensure a language dataset is available, downloading if necessary.
    ///
    /// This is the main entry point for getting data. It will:
    /// 1. Check if the language is already in the store
    /// 2. If not, download from GitHub and import
    ///
    /// Returns `true` if the dataset was downloaded, `false` if it was already cached.
    #[instrument(level = "info", skip(self))]
    pub async fn ensure(&mut self, lang: &str) -> Result<bool> {
        let lang_code = LangCode::new(lang)?;

        if self.store.has_language(lang)? {
            debug!(lang, "language already cached");
            return Ok(false);
        }

        info!(lang, "downloading language dataset");
        self.download_and_import(&lang_code).await?;
        Ok(true)
    }

    /// Force re-download and import a language dataset.
    ///
    /// This will download the latest data from GitHub even if the language
    /// is already in the store.
    #[instrument(level = "info", skip(self))]
    pub async fn refresh(&mut self, lang: &str) -> Result<()> {
        let lang_code = LangCode::new(lang)?;
        info!(lang, "refreshing language dataset");
        self.download_and_import(&lang_code).await
    }

    /// Download and import a language dataset.
    #[instrument(level = "debug", skip(self))]
    async fn download_and_import(&mut self, lang: &LangCode) -> Result<()> {
        let content = download_language(lang).await?;
        let (entries, skipped) = Entry::parse_tsv_lenient(&content);

        if skipped > 0 {
            warn!(
                lang = %lang,
                skipped,
                "skipped malformed entries during import"
            );
        }

        debug!(
            lang = %lang,
            entries = entries.len(),
            "parsed entries from downloaded data"
        );

        let source_url = format!("https://github.com/unimorph/{}", lang.as_str());

        self.store.import(lang, &entries, Some(&source_url))?;
        info!(
            lang = %lang,
            entries = entries.len(),
            "imported language dataset"
        );
        Ok(())
    }

    /// List all languages available in the local store.
    pub fn cached_languages(&self) -> Result<Vec<LangCode>> {
        self.store.languages()
    }

    /// Delete a language from the local store.
    pub fn delete(&mut self, lang: &str) -> Result<()> {
        self.store.delete_language(lang)
    }
}

/// Get the file patterns to download for a language.
///
/// Most languages have a single file named after the language code,
/// but some (like Finnish) have multiple files.
fn get_file_patterns(lang: &LangCode) -> Vec<String> {
    match lang.as_str() {
        // Languages known to have split files
        "fin" => vec!["fin.1".to_string(), "fin.2".to_string()],
        // Default: single file named after the language code
        _ => vec![lang.as_str().to_string()],
    }
}

/// Download a language dataset from GitHub.
#[instrument(level = "debug")]
async fn download_language(lang: &LangCode) -> Result<String> {
    let client = reqwest::Client::new();
    let patterns = get_file_patterns(lang);
    let mut all_content = String::new();
    let mut found_any = false;

    debug!(lang = %lang, patterns = ?patterns, "downloading from GitHub");

    for pattern in &patterns {
        let url = format!("{}/{}/master/{}", UNIMORPH_RAW_URL, lang.as_str(), pattern);

        debug!(url = %url, "fetching file");
        let response = client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::FORBIDDEN {
            warn!(lang = %lang, "GitHub rate limit exceeded");
            return Err(Error::RateLimited);
        }

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!(url = %url, "file not found, trying next pattern");
            continue;
        }

        if !response.status().is_success() {
            return Err(Error::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let content = response.text().await?;
        let bytes = content.len();
        debug!(url = %url, bytes, "downloaded file");
        all_content.push_str(&content);
        if !content.ends_with('\n') {
            all_content.push('\n');
        }
        found_any = true;
    }

    if !found_any {
        return Err(Error::DownloadFailed(format!(
            "No data files found for language: {}",
            lang.as_str()
        )));
    }

    Ok(all_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn repository_with_custom_dir() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        assert!(repo.cache_dir().exists());
        assert!(repo.cache_dir().join("datasets.db").exists());
    }

    #[test]
    fn cached_languages_empty() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let langs = repo.cached_languages().unwrap();
        assert!(langs.is_empty());
    }

    #[test]
    fn file_patterns() {
        let ita: LangCode = "ita".parse().unwrap();
        let fin: LangCode = "fin".parse().unwrap();

        assert_eq!(get_file_patterns(&ita), vec!["ita"]);
        assert_eq!(get_file_patterns(&fin), vec!["fin.1", "fin.2"]);
    }

    // Integration tests that require network would go here with #[ignore]
    // #[tokio::test]
    // #[ignore]
    // async fn download_italian() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let mut repo = Repository::with_cache_dir(temp_dir.path()).unwrap();
    //
    //     let downloaded = repo.ensure("ita").await.unwrap();
    //     assert!(downloaded);
    //
    //     let downloaded_again = repo.ensure("ita").await.unwrap();
    //     assert!(!downloaded_again); // Should be cached
    // }
}
