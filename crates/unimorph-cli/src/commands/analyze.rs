//! Analyze command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{debug, instrument};

use crate::util::{create_repo, require_language, validate_lang_code};

#[instrument(skip_all, fields(lang, form))]
pub fn cmd_analyze(lang: &str, form: &str, json: bool, data_dir: Option<&Path>) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let entries = repo.store().analyze(lang, form)?;

    debug!(count = entries.len(), "found analyses");

    if entries.is_empty() {
        println!("No analyses found for '{}'.", form);
        println!();
        println!("The form may not exist in the dataset, or it could be:");
        println!("  - A proper noun or foreign word");
        println!("  - A misspelling");
        println!("  - A rare or archaic form");
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        println!("{:<20} {:<20} FEATURES", "FORM", "LEMMA");
        println!("{}", "-".repeat(60));
        for entry in &entries {
            println!("{:<20} {:<20} {}", entry.form, entry.lemma, entry.features);
        }
        println!();
        println!("{} analysis(es) found.", entries.len());
    }

    Ok(())
}
