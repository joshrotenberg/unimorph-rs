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

use std::io::Read;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tracing::{debug, info, instrument, warn};

use crate::{Entry, Error, LangCode, Result, Store};

/// Phase of the download/import operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadPhase {
    /// Downloading data from GitHub.
    Downloading,
    /// Parsing TSV and importing into SQLite.
    Importing,
}

/// Progress information for download operations.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Current phase of the operation.
    pub phase: DownloadPhase,
    /// Total bytes expected (if known from Content-Length header).
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Current file being downloaded (for multi-file languages like Finnish).
    pub current_file: String,
    /// Total number of files to download.
    pub total_files: usize,
    /// Current file index (1-based).
    pub current_file_index: usize,
}

const UNIMORPH_RAW_URL: &str = "https://raw.githubusercontent.com/unimorph";

/// Decompress content based on file extension.
///
/// Supports `.xz` (LZMA), `.gz` (gzip), and `.zip` formats.
/// Plain text files are returned as-is after UTF-8 conversion.
fn decompress_content(filename: &str, bytes: &[u8]) -> Result<String> {
    if filename.ends_with(".xz") {
        debug!(filename, "decompressing XZ/LZMA content");
        let mut decoder = xz2::read::XzDecoder::new(bytes);
        let mut content = String::new();
        decoder
            .read_to_string(&mut content)
            .map_err(|e| Error::DecompressionFailed(format!("XZ decompression failed: {}", e)))?;
        Ok(content)
    } else if filename.ends_with(".gz") {
        debug!(filename, "decompressing gzip content");
        let mut decoder = flate2::read::GzDecoder::new(bytes);
        let mut content = String::new();
        decoder
            .read_to_string(&mut content)
            .map_err(|e| Error::DecompressionFailed(format!("gzip decompression failed: {}", e)))?;
        Ok(content)
    } else if filename.ends_with(".zip") {
        debug!(filename, "extracting ZIP content");
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| Error::DecompressionFailed(format!("ZIP archive error: {}", e)))?;
        if archive.is_empty() {
            return Err(Error::DecompressionFailed(
                "ZIP archive is empty".to_string(),
            ));
        }
        let mut file = archive
            .by_index(0)
            .map_err(|e| Error::DecompressionFailed(format!("ZIP extraction error: {}", e)))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| Error::DecompressionFailed(format!("ZIP read error: {}", e)))?;
        Ok(content)
    } else {
        // Plain text - convert from UTF-8
        String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::DecompressionFailed(format!("UTF-8 conversion failed: {}", e)))
    }
}

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

    /// Force re-download and import with progress reporting.
    ///
    /// The callback receives `DownloadProgress` updates during the download.
    #[instrument(level = "info", skip(self, on_progress))]
    pub async fn refresh_with_progress<F>(&mut self, lang: &str, on_progress: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let lang_code = LangCode::new(lang)?;
        info!(lang, "refreshing language dataset with progress");
        self.download_and_import_with_progress(&lang_code, on_progress)
            .await
    }

    /// Ensure a language is available, with progress reporting.
    ///
    /// Like `ensure`, but calls `on_progress` with download progress updates.
    /// Returns `true` if the dataset was downloaded, `false` if it was already cached.
    #[instrument(level = "info", skip(self, on_progress))]
    pub async fn ensure_with_progress<F>(&mut self, lang: &str, on_progress: F) -> Result<bool>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let lang_code = LangCode::new(lang)?;

        if self.store.has_language(lang)? {
            debug!(lang, "language already cached");
            return Ok(false);
        }

        info!(lang, "downloading language dataset with progress");
        self.download_and_import_with_progress(&lang_code, on_progress)
            .await?;
        Ok(true)
    }

    /// Download and import a language dataset.
    #[instrument(level = "debug", skip(self))]
    async fn download_and_import(&mut self, lang: &LangCode) -> Result<()> {
        // Fetch commit SHA first
        let commit_sha = fetch_commit_sha(lang).await.ok();
        debug!(lang = %lang, commit_sha = ?commit_sha, "fetched commit SHA");

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

        self.store
            .import(lang, &entries, Some(&source_url), commit_sha.as_deref())?;
        info!(
            lang = %lang,
            entries = entries.len(),
            commit_sha = ?commit_sha,
            "imported language dataset"
        );
        Ok(())
    }

    /// Download and import a language dataset with progress reporting.
    #[instrument(level = "debug", skip(self, on_progress))]
    async fn download_and_import_with_progress<F>(
        &mut self,
        lang: &LangCode,
        on_progress: F,
    ) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        // Fetch commit SHA first
        let commit_sha = fetch_commit_sha(lang).await.ok();
        debug!(lang = %lang, commit_sha = ?commit_sha, "fetched commit SHA");

        let content = download_language_with_progress(lang, &on_progress).await?;

        // Signal import phase
        on_progress(DownloadProgress {
            phase: DownloadPhase::Importing,
            total_bytes: None,
            downloaded_bytes: 0,
            current_file: String::new(),
            total_files: 0,
            current_file_index: 0,
        });

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

        self.store
            .import(lang, &entries, Some(&source_url), commit_sha.as_deref())?;
        info!(
            lang = %lang,
            entries = entries.len(),
            commit_sha = ?commit_sha,
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

/// Get the file pattern alternatives to try for a language.
///
/// Returns a list of alternatives to try in order. Each alternative is a list of
/// files that together make up the complete dataset. For most languages, this is
/// a single file, but some (like Finnish) have multiple files.
///
/// Compressed versions (.xz, .gz) are tried first since they may contain more
/// complete data when the uncompressed version exceeds GitHub's file size limits.
fn get_file_alternatives(lang: &LangCode) -> Vec<Vec<String>> {
    match lang.as_str() {
        // Languages known to have split files
        "fin" => vec![vec!["fin.1".to_string(), "fin.2".to_string()]],
        // Default: try compressed versions first, then uncompressed
        _ => vec![
            vec![format!("{}.xz", lang.as_str())],
            vec![format!("{}.gz", lang.as_str())],
            vec![lang.as_str().to_string()],
        ],
    }
}

/// Download a language dataset from GitHub.
///
/// Tries multiple file alternatives in order (compressed first, then uncompressed).
/// Automatically decompresses `.xz`, `.gz`, and `.zip` files.
#[instrument(level = "debug")]
async fn download_language(lang: &LangCode) -> Result<String> {
    let client = reqwest::Client::new();
    let alternatives = get_file_alternatives(lang);

    debug!(lang = %lang, alternatives = ?alternatives, "downloading from GitHub");

    // Try each alternative in order
    for files in &alternatives {
        match try_download_files(&client, lang, files).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                debug!(files = ?files, error = %e, "alternative failed, trying next");
                continue;
            }
        }
    }

    Err(Error::DownloadFailed(format!(
        "No data files found for language: {}",
        lang.as_str()
    )))
}

/// Try to download a specific set of files for a language.
///
/// Returns the concatenated, decompressed content if all files are found.
/// Returns an error if any file is not found or fails to download.
async fn try_download_files(
    client: &reqwest::Client,
    lang: &LangCode,
    files: &[String],
) -> Result<String> {
    let mut all_content = String::new();

    for filename in files {
        let url = format!("{}/{}/master/{}", UNIMORPH_RAW_URL, lang.as_str(), filename);

        debug!(url = %url, "fetching file");
        let response = client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::FORBIDDEN {
            warn!(lang = %lang, "GitHub rate limit exceeded");
            return Err(Error::RateLimited);
        }

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!(url = %url, "file not found");
            return Err(Error::DownloadFailed(format!("File not found: {}", url)));
        }

        if !response.status().is_success() {
            return Err(Error::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        // Download as bytes for decompression
        let bytes = response.bytes().await?;
        debug!(url = %url, bytes = bytes.len(), "downloaded file");

        // Decompress based on file extension
        let content = decompress_content(filename, &bytes)?;

        all_content.push_str(&content);
        if !content.ends_with('\n') {
            all_content.push('\n');
        }
    }

    Ok(all_content)
}

/// Download a language dataset from GitHub with progress reporting.
///
/// Tries multiple file alternatives in order (compressed first, then uncompressed).
/// Automatically decompresses `.xz`, `.gz`, and `.zip` files.
#[instrument(level = "debug", skip(on_progress))]
async fn download_language_with_progress<F>(lang: &LangCode, on_progress: &F) -> Result<String>
where
    F: Fn(DownloadProgress) + Send + Sync,
{
    let client = reqwest::Client::new();
    let alternatives = get_file_alternatives(lang);

    debug!(lang = %lang, alternatives = ?alternatives, "downloading from GitHub with progress");

    // Try each alternative in order
    for files in &alternatives {
        match try_download_files_with_progress(&client, lang, files, on_progress).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                debug!(files = ?files, error = %e, "alternative failed, trying next");
                continue;
            }
        }
    }

    Err(Error::DownloadFailed(format!(
        "No data files found for language: {}",
        lang.as_str()
    )))
}

/// Try to download a specific set of files for a language with progress reporting.
///
/// Returns the concatenated, decompressed content if all files are found.
/// Returns an error if any file is not found or fails to download.
async fn try_download_files_with_progress<F>(
    client: &reqwest::Client,
    lang: &LangCode,
    files: &[String],
    on_progress: &F,
) -> Result<String>
where
    F: Fn(DownloadProgress) + Send + Sync,
{
    let total_files = files.len();
    let mut all_content = String::new();

    for (file_index, filename) in files.iter().enumerate() {
        let url = format!("{}/{}/master/{}", UNIMORPH_RAW_URL, lang.as_str(), filename);

        debug!(url = %url, "fetching file");
        let response = client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::FORBIDDEN {
            warn!(lang = %lang, "GitHub rate limit exceeded");
            return Err(Error::RateLimited);
        }

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!(url = %url, "file not found");
            return Err(Error::DownloadFailed(format!("File not found: {}", url)));
        }

        if !response.status().is_success() {
            return Err(Error::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let total_bytes = response.content_length();
        let mut downloaded_bytes: u64 = 0;
        let mut bytes = Vec::new();

        // Send initial progress
        on_progress(DownloadProgress {
            phase: DownloadPhase::Downloading,
            total_bytes,
            downloaded_bytes,
            current_file: filename.clone(),
            total_files,
            current_file_index: file_index + 1,
        });

        // Stream the response body
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded_bytes += chunk.len() as u64;
            bytes.extend_from_slice(&chunk);

            on_progress(DownloadProgress {
                phase: DownloadPhase::Downloading,
                total_bytes,
                downloaded_bytes,
                current_file: filename.clone(),
                total_files,
                current_file_index: file_index + 1,
            });
        }

        debug!(url = %url, bytes = bytes.len(), "downloaded file");

        // Decompress based on file extension
        let content = decompress_content(filename, &bytes)?;

        all_content.push_str(&content);
        if !content.ends_with('\n') {
            all_content.push('\n');
        }
    }

    Ok(all_content)
}

/// Fetch the latest commit SHA for a language repository.
#[instrument(level = "debug")]
async fn fetch_commit_sha(lang: &LangCode) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/unimorph/{}/commits/master",
        lang.as_str()
    );

    debug!(url = %url, "fetching commit SHA");

    let response = client
        .get(&url)
        .header("User-Agent", "unimorph-rs")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(Error::RateLimited);
    }

    if !response.status().is_success() {
        return Err(Error::DownloadFailed(format!(
            "Failed to fetch commit info: HTTP {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response.json().await?;
    let sha = json["sha"]
        .as_str()
        .ok_or_else(|| Error::DownloadFailed("No SHA in commit response".to_string()))?
        .to_string();

    debug!(sha = %sha, "fetched commit SHA");
    Ok(sha)
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
    fn file_alternatives() {
        let ita: LangCode = "ita".parse().unwrap();
        let fin: LangCode = "fin".parse().unwrap();

        // Italian should try compressed first, then uncompressed
        let ita_alts = get_file_alternatives(&ita);
        assert_eq!(ita_alts.len(), 3);
        assert_eq!(ita_alts[0], vec!["ita.xz"]);
        assert_eq!(ita_alts[1], vec!["ita.gz"]);
        assert_eq!(ita_alts[2], vec!["ita"]);

        // Finnish has split files
        let fin_alts = get_file_alternatives(&fin);
        assert_eq!(fin_alts.len(), 1);
        assert_eq!(fin_alts[0], vec!["fin.1", "fin.2"]);
    }

    #[test]
    fn decompress_plain_text() {
        let content = b"lemma\tform\tfeatures\n";
        let result = decompress_content("test.txt", content).unwrap();
        assert_eq!(result, "lemma\tform\tfeatures\n");
    }

    #[test]
    fn decompress_gzip() {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let original = "lemma\tform\tV;IND;PRS\n";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let result = decompress_content("test.gz", &compressed).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn decompress_xz() {
        use std::io::Write;
        use xz2::write::XzEncoder;

        let original = "lemma\tform\tV;IND;PRS\n";
        let mut encoder = XzEncoder::new(Vec::new(), 6);
        encoder.write_all(original.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let result = decompress_content("test.xz", &compressed).unwrap();
        assert_eq!(result, original);
    }

    // Integration tests that require network access
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn download_italian_uncompressed() {
        // Italian uses uncompressed format
        let temp_dir = TempDir::new().unwrap();
        let mut repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let downloaded = repo.ensure("ita").await.unwrap();
        assert!(downloaded);

        // Verify data was imported
        let stats = repo.store().stats("ita").unwrap().unwrap();
        assert!(stats.total_entries > 0);

        // Second call should use cache
        let downloaded_again = repo.ensure("ita").await.unwrap();
        assert!(!downloaded_again);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn download_polish_compressed_xz() {
        // Polish uses .xz compression due to large file size
        let temp_dir = TempDir::new().unwrap();
        let mut repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let downloaded = repo.ensure("pol").await.unwrap();
        assert!(downloaded);

        // Verify data was imported
        let stats = repo.store().stats("pol").unwrap().unwrap();
        assert!(stats.total_entries > 0);
        // Polish is a large dataset
        assert!(stats.total_entries > 100_000);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn download_finnish_split_files() {
        // Finnish has split files (fin.1, fin.2)
        let temp_dir = TempDir::new().unwrap();
        let mut repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let downloaded = repo.ensure("fin").await.unwrap();
        assert!(downloaded);

        // Verify data was imported
        let stats = repo.store().stats("fin").unwrap().unwrap();
        assert!(stats.total_entries > 0);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn download_czech_small() {
        // Czech is a relatively small dataset, good for quick tests
        let temp_dir = TempDir::new().unwrap();
        let mut repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let downloaded = repo.ensure("ces").await.unwrap();
        assert!(downloaded);

        let stats = repo.store().stats("ces").unwrap().unwrap();
        assert!(stats.total_entries > 0);
    }
}
