//! Search command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{debug, instrument};

use crate::util::{create_repo, require_language, validate_lang_code};

#[instrument(skip_all, fields(lang))]
#[allow(clippy::too_many_arguments)]
pub fn cmd_search(
    lang: &str,
    lemma: Option<&str>,
    form: Option<&str>,
    features: Option<&str>,
    pos: Option<&str>,
    limit: usize,
    offset: Option<usize>,
    count: bool,
    json: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    let mut query = repo.store().query(lang);

    if let Some(l) = lemma {
        query = query.lemma(l);
    }
    if let Some(f) = form {
        query = query.form(f);
    }
    if let Some(feat) = features {
        query = query.features_match(feat);
    }
    if let Some(p) = pos {
        query = query.pos(p);
    }
    if let Some(off) = offset {
        query = query.offset(off);
    }
    query = query.limit(limit);

    if count {
        let n = query.count()?;
        if json {
            println!("{}", serde_json::json!({ "count": n }));
        } else {
            println!("{} entries match.", n);
        }
        return Ok(());
    }

    let entries = query.execute()?;

    debug!(count = entries.len(), "search results");

    if entries.is_empty() {
        println!("No entries match the search criteria.");
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        println!("{:<20} {:<20} FEATURES", "LEMMA", "FORM");
        println!("{}", "-".repeat(60));
        for entry in &entries {
            println!("{:<20} {:<20} {}", entry.lemma, entry.form, entry.features);
        }
        println!();
        println!("{} result(s).", entries.len());
    }

    Ok(())
}
