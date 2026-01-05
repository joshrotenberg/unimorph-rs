//! Info command implementation.

use std::path::Path;

use color_eyre::eyre::{Context, ContextCompat, Result};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::util::{create_repo, require_language, validate_lang_code};

/// Fetch the last pushed timestamp for a language repo from GitHub.
async fn fetch_repo_pushed_at(lang: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
    debug!(lang, "fetching repo info from GitHub");

    let octocrab = octocrab::instance();
    let repo = octocrab
        .repos("unimorph", lang)
        .get()
        .await
        .context("Failed to fetch repo info from GitHub")?;

    Ok(repo.pushed_at)
}

#[derive(Serialize)]
struct InfoOutput {
    language: String,
    source: String,
    imported_at: Option<String>,
    remote_updated_at: Option<String>,
    update_available: bool,
    stats: StatsOutput,
}

#[derive(Serialize)]
struct StatsOutput {
    total_entries: usize,
    unique_lemmas: usize,
    unique_forms: usize,
    unique_features: usize,
}

#[instrument(skip_all, fields(lang))]
pub async fn cmd_info(lang: &str, json: bool, data_dir: Option<&Path>) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let stats = repo
        .store()
        .stats(lang)?
        .context("Failed to retrieve statistics")?;

    let imported_at = repo.store().imported_at(lang)?;
    let source = format!("https://github.com/unimorph/{}", lang);

    // Fetch remote info
    let remote_pushed_at = fetch_repo_pushed_at(lang).await.ok().flatten();

    // Determine if update is available
    let update_available = match (&imported_at, &remote_pushed_at) {
        (Some(local), Some(remote)) => {
            // Parse local timestamp and compare
            if let Ok(local_dt) = chrono::DateTime::parse_from_rfc3339(local) {
                remote > &local_dt.with_timezone(&chrono::Utc)
            } else {
                false
            }
        }
        _ => false,
    };

    if json {
        let output = InfoOutput {
            language: lang.to_string(),
            source,
            imported_at: imported_at.clone(),
            remote_updated_at: remote_pushed_at.map(|dt| dt.to_rfc3339()),
            update_available,
            stats: StatsOutput {
                total_entries: stats.total_entries,
                unique_lemmas: stats.unique_lemmas,
                unique_forms: stats.unique_forms,
                unique_features: stats.unique_features,
            },
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Language: {}", lang);
        println!("Source: {}", source);
        println!();

        if let Some(ref local) = imported_at {
            println!("Local imported:  {}", local);
        }
        if let Some(remote) = remote_pushed_at {
            println!("Remote updated:  {}", remote.to_rfc3339());
        }

        if update_available {
            println!();
            println!("Status: UPDATE AVAILABLE");
            println!();
            println!("Run 'unimorph update -l {}' to update.", lang);
        } else {
            println!();
            println!("Status: Up to date");
        }

        println!();
        println!("Statistics:");
        println!("  Total entries:   {}", stats.total_entries);
        println!("  Unique lemmas:   {}", stats.unique_lemmas);
        println!("  Unique forms:    {}", stats.unique_forms);
        println!("  Unique features: {}", stats.unique_features);
    }

    Ok(())
}
