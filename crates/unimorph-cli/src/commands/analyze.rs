//! Analyze command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{debug, instrument};

use crate::colors::{
    dim_style, feature_style, form_style, header_style, lemma_style, number_style, should_colorize,
    styled,
};
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
        let color = should_colorize();
        println!(
            "{:<20} {:<20} {}",
            styled("FORM", header_style(), color),
            styled("LEMMA", header_style(), color),
            styled("FEATURES", header_style(), color)
        );
        println!("{}", styled("-".repeat(60), dim_style(), color));
        for entry in &entries {
            println!(
                "{:<20} {:<20} {}",
                styled(&entry.form, form_style(), color),
                styled(&entry.lemma, lemma_style(), color),
                styled(&entry.features, feature_style(), color)
            );
        }
        println!();
        println!(
            "{} analysis(es) found.",
            styled(entries.len(), number_style(), color)
        );
    }

    Ok(())
}
