//! Info command implementation.

use std::path::Path;

use chrono::{TimeZone, Utc};
use color_eyre::eyre::{Context, ContextCompat, Result};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::util::{create_repo, require_language, validate_lang_code};

/// Format a Unix timestamp string as a human-readable date.
fn format_timestamp(ts: &str) -> String {
    ts.parse::<i64>()
        .ok()
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

/// Fetch the latest commit info for a language repo from GitHub.
async fn fetch_remote_commit(lang: &str) -> Result<(String, chrono::DateTime<Utc>)> {
    debug!(lang, "fetching commit info from GitHub");

    let octocrab = octocrab::instance();
    let commits = octocrab
        .repos("unimorph", lang)
        .list_commits()
        .per_page(1)
        .send()
        .await
        .context("Failed to fetch commits from GitHub")?;

    let commit = commits
        .items
        .first()
        .context("No commits found in repository")?;

    let sha = commit.sha.clone();
    let date = commit
        .commit
        .committer
        .as_ref()
        .and_then(|c| c.date.as_ref())
        .context("No commit date found")?;

    debug!(sha = %sha, date = %date, "fetched remote commit info");
    Ok((sha, *date))
}

#[derive(Serialize)]
struct InfoOutput {
    language: String,
    source: String,
    imported_at: Option<String>,
    local_commit: Option<String>,
    remote_commit: Option<String>,
    remote_commit_date: Option<String>,
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
    let local_commit = repo.store().commit_sha(lang)?;
    let source = format!("https://github.com/unimorph/{}", lang);

    // Fetch remote commit info
    let remote_info = fetch_remote_commit(lang).await.ok();
    let (remote_commit, remote_date) = match remote_info {
        Some((sha, date)) => (Some(sha), Some(date)),
        None => (None, None),
    };

    // Determine if update is available by comparing commit SHAs
    let update_available = match (&local_commit, &remote_commit) {
        (Some(local), Some(remote)) => local != remote,
        (None, Some(_)) => true, // No local SHA stored, assume update available
        _ => false,
    };

    if json {
        let output = InfoOutput {
            language: lang.to_string(),
            source,
            imported_at: imported_at.as_ref().map(|ts| format_timestamp(ts)),
            local_commit,
            remote_commit: remote_commit.clone(),
            remote_commit_date: remote_date
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
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
            println!("Local imported:  {}", format_timestamp(local));
        }
        if let Some(ref sha) = local_commit {
            println!("Local commit:    {}", &sha[..7.min(sha.len())]);
        }
        if let Some(ref sha) = remote_commit {
            let date_str = remote_date
                .map(|d| d.format(" (%Y-%m-%d)").to_string())
                .unwrap_or_default();
            println!("Remote commit:   {}{}", &sha[..7.min(sha.len())], date_str);
        }

        println!();
        if update_available {
            println!("Status: UPDATE AVAILABLE");
            println!();
            println!("Run 'unimorph update {}' to update.", lang);
        } else {
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
