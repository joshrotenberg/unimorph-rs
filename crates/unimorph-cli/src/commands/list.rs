//! List command implementation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::util::create_repo;

/// Cached list of available languages from GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AvailableLanguagesCache {
    /// When the cache was last updated.
    updated_at: String,
    /// List of language codes.
    languages: Vec<String>,
}

/// Non-language repos in the unimorph org that should be excluded.
const EXCLUDED_REPOS: &[&str] = &[
    "unimorph.github.io",
    "unimorph",
    "analyzers",
    "ud-compatibility",
    "zxx", // special code for "no linguistic content"
];

/// Get the path to the available languages cache file.
fn available_languages_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("unimorph").join("available_languages.json"))
}

/// Load cached available languages if not too old (24 hours).
fn load_available_languages_cache() -> Option<AvailableLanguagesCache> {
    let path = available_languages_cache_path()?;
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    let cache: AvailableLanguagesCache = serde_json::from_str(&content).ok()?;

    // Check if cache is less than 24 hours old
    let updated = chrono::DateTime::parse_from_rfc3339(&cache.updated_at).ok()?;
    let age = chrono::Utc::now().signed_duration_since(updated);
    if age.num_hours() < 24 {
        debug!(
            age_hours = age.num_hours(),
            count = cache.languages.len(),
            "using cached available languages"
        );
        Some(cache)
    } else {
        debug!(age_hours = age.num_hours(), "cache expired");
        None
    }
}

/// Save available languages to cache.
fn save_available_languages_cache(languages: &[String]) -> Result<()> {
    let Some(path) = available_languages_cache_path() else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cache = AvailableLanguagesCache {
        updated_at: chrono::Utc::now().to_rfc3339(),
        languages: languages.to_vec(),
    };

    let content = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&path, content)?;
    debug!(path = %path.display(), "saved available languages cache");
    Ok(())
}

/// Fetch available languages from GitHub API.
async fn fetch_available_languages() -> Result<Vec<String>> {
    debug!("fetching available languages from GitHub API");

    let octocrab = octocrab::instance();
    let mut languages = Vec::new();
    let mut page = 1u32;

    loop {
        let repos = octocrab
            .orgs("unimorph")
            .list_repos()
            .per_page(100)
            .page(page)
            .send()
            .await
            .context("Failed to fetch repos from GitHub")?;

        if repos.items.is_empty() {
            break;
        }

        for repo in &repos.items {
            let name = &repo.name;
            // Filter: must be 3 lowercase letters and not in excluded list
            if name.len() == 3
                && name.chars().all(|c| c.is_ascii_lowercase())
                && !EXCLUDED_REPOS.contains(&name.as_str())
            {
                languages.push(name.clone());
            }
        }

        debug!(page, count = repos.items.len(), "fetched page of repos");

        if repos.next.is_none() {
            break;
        }
        page += 1;
    }

    languages.sort();
    debug!(total = languages.len(), "fetched available languages");
    Ok(languages)
}

pub async fn cmd_list(
    cached: bool,
    available: bool,
    refresh: bool,
    json: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    let repo = create_repo(data_dir)?;

    // Get cached languages
    let cached_langs: HashSet<String> = repo
        .cached_languages()?
        .into_iter()
        .map(|l| l.to_string())
        .collect();

    if cached && !available {
        // Show only cached languages
        if cached_langs.is_empty() {
            if json {
                println!("[]");
            } else {
                println!("No languages cached.");
                println!();
                println!("To download a language:");
                println!("  unimorph download <lang>");
                println!();
                println!("Examples:");
                println!("  unimorph download heb   # Hebrew");
                println!("  unimorph download vec   # Venetian");
            }
        } else if json {
            let langs: Vec<_> = cached_langs.iter().collect();
            println!("{}", serde_json::to_string_pretty(&langs)?);
        } else {
            println!("Cached languages:");
            let mut langs: Vec<_> = cached_langs.iter().collect();
            langs.sort();
            for lang in langs {
                let stats = repo.store().stats(lang)?;
                if let Some(stats) = stats {
                    println!("  {} ({} entries)", lang, stats.total_entries);
                } else {
                    println!("  {}", lang);
                }
            }
        }
        return Ok(());
    }

    if available || refresh {
        // Fetch or load available languages
        let available_langs = if refresh {
            let langs = fetch_available_languages().await?;
            save_available_languages_cache(&langs)?;
            langs
        } else if let Some(cache) = load_available_languages_cache() {
            cache.languages
        } else {
            let langs = fetch_available_languages().await?;
            if let Err(e) = save_available_languages_cache(&langs) {
                warn!(error = %e, "failed to save cache");
            }
            langs
        };

        if json {
            #[derive(Serialize)]
            struct LangInfo {
                code: String,
                cached: bool,
            }
            let langs: Vec<_> = available_langs
                .iter()
                .map(|code| LangInfo {
                    code: code.clone(),
                    cached: cached_langs.contains(code),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&langs)?);
        } else {
            println!(
                "Available languages ({} total, {} cached):",
                available_langs.len(),
                cached_langs.len()
            );
            println!();
            for lang in &available_langs {
                if cached_langs.contains(lang) {
                    println!("  {} [cached]", lang);
                } else {
                    println!("  {}", lang);
                }
            }
            println!();
            println!("Use 'unimorph download <code>' to download a language.");
            println!("Use 'unimorph list --refresh' to update this list.");
        }
        return Ok(());
    }

    // Default: show cached languages (or helpful info if none cached)
    if cached_langs.is_empty() {
        // No languages cached - show helpful info
        if json {
            println!("[]");
        } else {
            println!("No languages cached yet.");
            println!();
            println!("To download a language:");
            println!("  unimorph download <lang>");
            println!();
            println!("Examples:");
            println!("  unimorph download heb   # Hebrew");
            println!("  unimorph download vec   # Venetian");
            println!();
            println!("To see all available languages:");
            println!("  unimorph list --available");
            println!();
            println!("More info: https://github.com/unimorph");
        }
    } else if json {
        let langs: Vec<_> = cached_langs.iter().collect();
        println!("{}", serde_json::to_string_pretty(&langs)?);
    } else {
        println!("Cached languages:");
        let mut langs: Vec<_> = cached_langs.iter().collect();
        langs.sort();
        for lang in langs {
            let stats = repo.store().stats(lang)?;
            if let Some(stats) = stats {
                println!("  {} ({} entries)", lang, stats.total_entries);
            } else {
                println!("  {}", lang);
            }
        }
        println!();
        println!("Use 'unimorph list --available' to see all available languages.");
    }

    Ok(())
}
