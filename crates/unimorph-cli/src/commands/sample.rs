//! Sample command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{debug, instrument};

use crate::colors::{
    dim_style, feature_style, form_style, header_style, lemma_style, number_style, should_colorize,
    styled,
};
use crate::util::{create_repo, require_language, validate_lang_code};

/// Randomly sample entries from a language dataset.
#[instrument(skip_all, fields(lang, n))]
#[allow(clippy::too_many_arguments)]
pub fn cmd_sample(
    lang: &str,
    n: usize,
    seed: Option<u64>,
    by_lemma: bool,
    json: bool,
    tsv: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let store = repo.store();

    let entries = if by_lemma {
        store.sample_by_lemma(lang, n, seed)?
    } else {
        store.sample(lang, n, seed)?
    };

    debug!(count = entries.len(), "sampled entries");

    if entries.is_empty() {
        if !json && !tsv {
            println!("No entries to sample.");
        } else if json {
            println!("[]");
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
        let mode = if by_lemma { " (by lemma)" } else { "" };
        println!(
            "{} sampled entry(ies){}.",
            styled(entries.len(), number_style(), color),
            mode
        );
    }

    Ok(())
}
