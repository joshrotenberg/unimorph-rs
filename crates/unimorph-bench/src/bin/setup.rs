//! Setup binary for downloading UniMorph datasets and populating storage backends.
//!
//! Usage:
//!   cargo run --bin setup -- --langs ita,fin
//!   cargo run --bin setup -- --langs ita --backends sqlite,duckdb

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;

use unimorph_bench::{
    Entry, LangCode, Result, Store, duckdb::DuckDbStore, parquet::ParquetStore, sqlite::SqliteStore,
};

const UNIMORPH_RAW_URL: &str = "https://raw.githubusercontent.com/unimorph";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Backend {
    Sqlite,
    DuckDb,
    Parquet,
}

impl std::str::FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(Backend::Sqlite),
            "duckdb" => Ok(Backend::DuckDb),
            "parquet" => Ok(Backend::Parquet),
            _ => Err(format!("unknown backend: {}", s)),
        }
    }
}

struct Config {
    langs: Vec<LangCode>,
    backends: HashSet<Backend>,
    cache_dir: PathBuf,
    force: bool,
}

impl Config {
    fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        let mut langs = Vec::new();
        let mut backends = HashSet::new();
        let mut cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("unimorph-bench");
        let mut force = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--langs" | "-l" => {
                    i += 1;
                    if i < args.len() {
                        for code in args[i].split(',') {
                            langs.push(code.trim().parse()?);
                        }
                    }
                }
                "--backends" | "-b" => {
                    i += 1;
                    if i < args.len() {
                        for b in args[i].split(',') {
                            backends.insert(
                                b.trim().parse().map_err(|e: String| {
                                    unimorph_bench::Error::DownloadFailed(e)
                                })?,
                            );
                        }
                    }
                }
                "--cache-dir" | "-c" => {
                    i += 1;
                    if i < args.len() {
                        cache_dir = PathBuf::from(&args[i]);
                    }
                }
                "--force" | "-f" => {
                    force = true;
                }
                "--help" | "-h" => {
                    println!("Usage: setup [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!(
                        "  -l, --langs <CODES>     Comma-separated language codes (e.g., ita,fin)"
                    );
                    println!(
                        "  -b, --backends <LIST>   Comma-separated backends: sqlite,duckdb,parquet"
                    );
                    println!(
                        "  -c, --cache-dir <PATH>  Cache directory (default: ~/.cache/unimorph-bench)"
                    );
                    println!("  -f, --force             Re-download even if cached");
                    println!("  -h, --help              Show this help");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[i]);
                    std::process::exit(1);
                }
            }
            i += 1;
        }

        // Defaults
        if langs.is_empty() {
            langs.push("ita".parse()?);
        }
        if backends.is_empty() {
            backends.insert(Backend::Sqlite);
            backends.insert(Backend::DuckDb);
            backends.insert(Backend::Parquet);
        }

        Ok(Self {
            langs,
            backends,
            cache_dir,
            force,
        })
    }
}

/// Get the file patterns to try for a language.
/// Some languages have split files (e.g., fin.1, fin.2) while others have a single file.
fn get_file_patterns(lang: &LangCode) -> Vec<String> {
    match lang.as_str() {
        // Languages known to have split files
        "fin" => vec!["fin.1".to_string(), "fin.2".to_string()],
        // Default: try single file named after the language code
        _ => vec![lang.as_str().to_string()],
    }
}

async fn download_file(client: &reqwest::Client, url: &str) -> Result<Option<String>> {
    let response = client.get(url).send().await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(unimorph_bench::Error::RateLimited);
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(unimorph_bench::Error::DownloadFailed(format!(
            "HTTP {}: {}",
            response.status(),
            url
        )));
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("valid template")
            .progress_chars("#>-"),
    );

    let content = response.text().await?;
    pb.finish_with_message("done");

    Ok(Some(content))
}

async fn download_dataset(lang: &LangCode, cache_dir: &PathBuf, force: bool) -> Result<Vec<Entry>> {
    let tsv_path = cache_dir.join(format!("{}.tsv", lang.as_str()));

    // Check cache
    if tsv_path.exists() && !force {
        println!("Using cached {}", tsv_path.display());
        let content = tokio::fs::read_to_string(&tsv_path).await?;
        return Entry::parse_tsv(&content);
    }

    std::fs::create_dir_all(cache_dir)?;
    let client = reqwest::Client::new();

    let patterns = get_file_patterns(lang);
    let mut all_content = String::new();
    let mut found_any = false;

    for pattern in &patterns {
        let url = format!("{}/{}/master/{}", UNIMORPH_RAW_URL, lang.as_str(), pattern);
        println!("Downloading {}...", url);

        match download_file(&client, &url).await? {
            Some(content) => {
                all_content.push_str(&content);
                if !content.ends_with('\n') {
                    all_content.push('\n');
                }
                found_any = true;
            }
            None => {
                println!("  Not found, skipping");
            }
        }
    }

    if !found_any {
        return Err(unimorph_bench::Error::DownloadFailed(format!(
            "No data files found for language: {}",
            lang.as_str()
        )));
    }

    // Cache the combined TSV
    let mut file = tokio::fs::File::create(&tsv_path).await?;
    file.write_all(all_content.as_bytes()).await?;
    println!("Cached to {}", tsv_path.display());

    // Use lenient parsing - some datasets have malformed entries
    let (entries, skipped) = Entry::parse_tsv_lenient(&all_content);
    if skipped > 0 {
        println!("  (skipped {} malformed entries)", skipped);
    }
    Ok(entries)
}

async fn populate_backends(
    entries: &[Entry],
    lang: &LangCode,
    backends: &HashSet<Backend>,
    cache_dir: &Path,
) -> Result<()> {
    if backends.contains(&Backend::Sqlite) {
        let db_path = cache_dir.join("unimorph.sqlite");
        println!("Populating SQLite: {}", db_path.display());
        let mut store = SqliteStore::open(&db_path)?;
        store.init(lang, entries)?;
        let size = std::fs::metadata(&db_path)?.len();
        println!(
            "  SQLite size: {} bytes ({:.2} MB)",
            size,
            size as f64 / 1_000_000.0
        );
    }

    if backends.contains(&Backend::DuckDb) {
        let db_path = cache_dir.join("unimorph.duckdb");
        println!("Populating DuckDB: {}", db_path.display());
        let mut store = DuckDbStore::open(&db_path)?;
        store.init(lang, entries)?;
        store.create_indexes()?;
        let size = std::fs::metadata(&db_path)?.len();
        println!(
            "  DuckDB size: {} bytes ({:.2} MB)",
            size,
            size as f64 / 1_000_000.0
        );
    }

    if backends.contains(&Backend::Parquet) {
        let parquet_dir = cache_dir.join("parquet");
        println!("Populating Parquet: {}", parquet_dir.display());
        let mut store = ParquetStore::new(parquet_dir.clone())?;
        store.init(lang, entries)?;
        let parquet_path = parquet_dir.join(format!("{}.parquet", lang.as_str()));
        let size = std::fs::metadata(&parquet_path)?.len();
        println!(
            "  Parquet size: {} bytes ({:.2} MB)",
            size,
            size as f64 / 1_000_000.0
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse()?;

    println!("UniMorph Benchmark Setup");
    println!("========================");
    println!("Cache directory: {}", config.cache_dir.display());
    println!(
        "Languages: {}",
        config
            .langs
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();

    std::fs::create_dir_all(&config.cache_dir)?;

    for lang in &config.langs {
        println!("\n--- {} ---", lang);

        let entries = download_dataset(lang, &config.cache_dir, config.force).await?;
        println!("Loaded {} entries", entries.len());

        populate_backends(&entries, lang, &config.backends, &config.cache_dir).await?;
    }

    println!("\nSetup complete!");
    Ok(())
}
