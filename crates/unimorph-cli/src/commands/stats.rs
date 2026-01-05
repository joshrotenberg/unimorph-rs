//! Stats command implementation.

use std::path::Path;

use color_eyre::eyre::{ContextCompat, Result};
use tracing::instrument;

use crate::util::{create_repo, require_language, validate_lang_code};

#[instrument(skip_all, fields(lang))]
pub fn cmd_stats(lang: &str, json: bool, data_dir: Option<&Path>) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let stats = repo
        .store()
        .stats(lang)?
        .context("Failed to retrieve statistics")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Statistics for {}:", lang);
        println!("  Total entries:    {}", stats.total_entries);
        println!("  Unique lemmas:    {}", stats.unique_lemmas);
        println!("  Unique forms:     {}", stats.unique_forms);
        println!("  Unique features:  {}", stats.unique_features);

        if let Some(imported_at) = repo.store().imported_at(lang)? {
            println!("  Imported at:      {}", imported_at);
        }
    }

    Ok(())
}
