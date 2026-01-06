//! Inflect command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{debug, instrument};

use crate::colors::{
    dim_style, feature_style, form_style, header_style, lemma_style, number_style, should_colorize,
    styled,
};
use crate::util::{create_repo, require_language, validate_lang_code};

#[instrument(skip_all, fields(lang, lemma))]
pub fn cmd_inflect(
    lang: &str,
    lemma: &str,
    features: Option<&str>,
    json: bool,
    tsv: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let entries = repo.store().inflect(lang, lemma)?;

    // Filter by features if specified
    let entries: Vec<_> = if let Some(pattern) = features {
        entries
            .into_iter()
            .filter(|e| e.features.matches_pattern(pattern))
            .collect()
    } else {
        entries
    };

    debug!(count = entries.len(), "found forms");

    if entries.is_empty() {
        if features.is_some() {
            println!(
                "No forms found for '{}' matching the feature pattern.",
                lemma
            );
            println!();
            println!(
                "Tip: Use 'unimorph inflect -l {} {}' without --features to see all forms.",
                lang, lemma
            );
        } else {
            println!("No forms found for '{}'.", lemma);
            println!();
            println!("The lemma may not exist in the dataset, or it might be spelled differently.");
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else if tsv {
        // TSV output: tab-separated, no headers, no summary - ideal for piping
        for entry in &entries {
            println!("{}\t{}\t{}", entry.lemma, entry.form, entry.features);
        }
    } else {
        let color = should_colorize();
        println!(
            "{:<20} {:<20} {}",
            styled("LEMMA", header_style(), color),
            styled("FORM", header_style(), color),
            styled("FEATURES", header_style(), color)
        );
        println!("{}", styled("-".repeat(60), dim_style(), color));
        for entry in &entries {
            println!(
                "{:<20} {:<20} {}",
                styled(&entry.lemma, lemma_style(), color),
                styled(&entry.form, form_style(), color),
                styled(&entry.features, feature_style(), color)
            );
        }
        println!();
        println!(
            "{} form(s) found.",
            styled(entries.len(), number_style(), color)
        );
    }

    Ok(())
}
