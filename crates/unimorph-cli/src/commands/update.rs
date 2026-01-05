//! Update command implementation.

use std::io::IsTerminal;
use std::path::Path;

use color_eyre::eyre::{Context, ContextCompat, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::util::{create_repo, validate_lang_code};

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

/// Check if a language needs updating.
async fn check_update_needed(
    lang: &str,
    imported_at: Option<&str>,
) -> Result<(bool, Option<chrono::DateTime<chrono::Utc>>)> {
    let remote_pushed_at = fetch_repo_pushed_at(lang).await?;

    let needs_update = match (imported_at, &remote_pushed_at) {
        (Some(local), Some(remote)) => {
            if let Ok(local_dt) = chrono::DateTime::parse_from_rfc3339(local) {
                remote > &local_dt.with_timezone(&chrono::Utc)
            } else {
                true // Can't parse local, assume update needed
            }
        }
        (None, Some(_)) => true, // No local timestamp, assume update needed
        (_, None) => false,      // Can't fetch remote, skip
    };

    Ok((needs_update, remote_pushed_at))
}

#[derive(Serialize)]
struct UpdateCheckResult {
    language: String,
    needs_update: bool,
    local_version: Option<String>,
    remote_version: Option<String>,
}

#[derive(Serialize)]
struct UpdateResult {
    language: String,
    updated: bool,
    entries: Option<usize>,
    error: Option<String>,
}

#[instrument(skip_all, fields(lang))]
pub async fn cmd_update(
    lang: Option<&str>,
    all: bool,
    check_only: bool,
    json: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    let mut repo = create_repo(data_dir)?;

    // Determine which languages to update
    let languages: Vec<String> = if all {
        repo.cached_languages()?
            .into_iter()
            .map(|l| l.to_string())
            .collect()
    } else if let Some(l) = lang {
        validate_lang_code(l)?;
        if !repo.store().has_language(l)? {
            if json {
                let result = UpdateResult {
                    language: l.to_string(),
                    updated: false,
                    entries: None,
                    error: Some(format!("Language '{}' is not cached", l)),
                };
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Language '{}' is not cached.", l);
                println!();
                println!("To download it first, run:");
                println!("  unimorph download -l {}", l);
            }
            return Ok(());
        }
        vec![l.to_string()]
    } else {
        if json {
            println!("[]");
        } else {
            println!("No language specified.");
            println!();
            println!("Usage:");
            println!("  unimorph update -l <lang>   Update a specific language");
            println!("  unimorph update --all       Update all cached languages");
            println!("  unimorph update --check     Check for updates without downloading");
        }
        return Ok(());
    };

    if languages.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No languages cached.");
            println!();
            println!("To download a language:");
            println!("  unimorph download -l heb");
        }
        return Ok(());
    }

    if check_only {
        // Just check for updates
        let mut results = Vec::new();

        if !json {
            println!("Checking for updates...");
            println!();
        }

        for lang in &languages {
            let imported_at = repo.store().imported_at(lang)?;
            let (needs_update, remote_pushed_at) =
                check_update_needed(lang, imported_at.as_deref()).await?;

            if json {
                results.push(UpdateCheckResult {
                    language: lang.clone(),
                    needs_update,
                    local_version: imported_at,
                    remote_version: remote_pushed_at.map(|dt| dt.to_rfc3339()),
                });
            } else if needs_update {
                println!("  {} - update available", lang);
            } else {
                println!("  {} - up to date", lang);
            }
        }

        if json {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }

        return Ok(());
    }

    // Actually perform updates
    let mut results = Vec::new();
    let mut updated_count = 0;

    let is_terminal = std::io::stdout().is_terminal();

    for lang in &languages {
        let imported_at = repo.store().imported_at(lang)?;
        let (needs_update, _) = check_update_needed(lang, imported_at.as_deref()).await?;

        if !needs_update {
            if !json {
                println!("  {} - already up to date", lang);
            }
            results.push(UpdateResult {
                language: lang.clone(),
                updated: false,
                entries: None,
                error: None,
            });
            continue;
        }

        let pb = if !json && is_terminal {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .expect("valid template"),
            );
            pb.set_message(format!("Updating {}...", lang));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        match repo.refresh(lang).await {
            Ok(()) => {
                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }

                let stats = repo
                    .store()
                    .stats(lang)?
                    .context("Failed to retrieve stats after update")?;

                info!(lang, entries = stats.total_entries, "update complete");

                if !json {
                    println!("  {} - updated ({} entries)", lang, stats.total_entries);
                }

                results.push(UpdateResult {
                    language: lang.clone(),
                    updated: true,
                    entries: Some(stats.total_entries),
                    error: None,
                });
                updated_count += 1;
            }
            Err(e) => {
                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }

                if !json {
                    println!("  {} - error: {}", lang, e);
                }

                results.push(UpdateResult {
                    language: lang.clone(),
                    updated: false,
                    entries: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!();
        println!(
            "Done. {} of {} language(s) updated.",
            updated_count,
            languages.len()
        );
    }

    Ok(())
}
