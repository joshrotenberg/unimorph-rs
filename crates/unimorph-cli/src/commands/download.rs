//! Download command implementation.

use std::io::IsTerminal;
use std::path::Path;

use color_eyre::eyre::{Context, ContextCompat, Result};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{info, instrument};

use crate::util::{create_repo, validate_lang_code};

#[instrument(skip_all, fields(lang, force))]
pub async fn cmd_download(
    lang: &str,
    force: bool,
    quiet: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = create_repo(data_dir)?;

    let is_terminal = std::io::stdout().is_terminal();
    let pb = if !quiet && is_terminal {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("valid template"),
        );
        pb.set_message(format!("Downloading {}...", lang));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let downloaded = if force {
        repo.refresh(lang)
            .await
            .context(format!("Failed to download '{}'", lang))?;
        true
    } else {
        repo.ensure(lang)
            .await
            .context(format!("Failed to download '{}'", lang))?
    };

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    if downloaded {
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
        if !quiet {
            println!(
                "Downloaded {}: {} entries, {} lemmas, {} forms",
                lang, stats.total_entries, stats.unique_lemmas, stats.unique_forms
            );
        }
    } else if !quiet {
        println!("{} is already cached. Use --force to re-download.", lang);
    }

    Ok(())
}
