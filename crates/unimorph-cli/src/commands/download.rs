//! Download command implementation.

use std::io::IsTerminal;
use std::path::Path;
use std::sync::{Arc, Mutex};

use color_eyre::eyre::{Context, ContextCompat, Result};
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use tracing::{info, instrument};
use unimorph_core::{DownloadPhase, DownloadProgress};

use crate::util::{create_repo, validate_lang_code};

#[instrument(skip_all, fields(lang, force))]
pub async fn cmd_download(
    lang: &str,
    force: bool,
    json: bool,
    quiet: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = create_repo(data_dir)?;

    let is_terminal = std::io::stdout().is_terminal();

    // Check if already cached (for non-force downloads)
    if !force && repo.store().has_language(lang)? {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "lang": lang,
                    "status": "cached",
                    "message": "Already cached. Use --force to re-download."
                })
            );
        } else if !quiet {
            println!("{} is already cached. Use --force to re-download.", lang);
        }
        return Ok(());
    }

    // Track total downloaded bytes across all files
    let total_downloaded = Arc::new(Mutex::new(0u64));

    let pb = if !quiet && is_terminal {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} [{bar:30.cyan/blue}] {bytes}/{total_bytes}")
                .expect("valid template")
                .progress_chars("=> "),
        );
        pb.set_message(format!("Downloading {}...", lang));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(Arc::new(pb))
    } else {
        None
    };

    let pb_clone = pb.clone();
    let total_downloaded_clone = total_downloaded.clone();
    let progress_callback = move |progress: DownloadProgress| {
        if let Some(ref pb) = pb_clone {
            match progress.phase {
                DownloadPhase::Downloading => {
                    // Update total if known
                    if let Some(total) = progress.total_bytes {
                        pb.set_length(total);
                    }
                    pb.set_position(progress.downloaded_bytes);

                    // Track total downloaded
                    if let Ok(mut total) = total_downloaded_clone.lock() {
                        *total = (*total).max(progress.downloaded_bytes);
                    }

                    // Update message for multi-file downloads
                    if progress.total_files > 1 {
                        pb.set_message(format!(
                            "Downloading {} ({}/{})",
                            progress.current_file,
                            progress.current_file_index,
                            progress.total_files
                        ));
                    }
                }
                DownloadPhase::Importing => {
                    // Switch to spinner for import phase
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .template("{spinner:.green} {msg}")
                            .expect("valid template"),
                    );
                    pb.set_message("Importing...");
                }
            }
        }
    };

    if force {
        repo.refresh_with_progress(lang, progress_callback)
            .await
            .context(format!("Failed to download '{}'", lang))?;
    } else {
        repo.ensure_with_progress(lang, progress_callback)
            .await
            .context(format!("Failed to download '{}'", lang))?;
    };

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    let bytes_downloaded = total_downloaded.lock().map(|t| *t).unwrap_or(0);

    let stats = repo
        .store()
        .stats(lang)?
        .context("Failed to retrieve stats after download")?;

    info!(
        lang,
        entries = stats.total_entries,
        lemmas = stats.unique_lemmas,
        forms = stats.unique_forms,
        "download complete"
    );

    if json {
        println!(
            "{}",
            serde_json::json!({
                "lang": lang,
                "status": "downloaded",
                "bytes": bytes_downloaded,
                "entries": stats.total_entries,
                "lemmas": stats.unique_lemmas,
                "forms": stats.unique_forms
            })
        );
    } else if !quiet {
        if bytes_downloaded > 0 {
            println!("Downloaded {}", HumanBytes(bytes_downloaded));
        }
        println!(
            "{}: {} entries, {} lemmas, {} forms",
            lang, stats.total_entries, stats.unique_lemmas, stats.unique_forms
        );
    }

    Ok(())
}
