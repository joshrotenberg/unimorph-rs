//! Delete command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::util::{create_repo, validate_lang_code};

#[instrument(skip_all, fields(lang))]
pub fn cmd_delete(lang: &str, data_dir: Option<&Path>) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = create_repo(data_dir)?;

    if !repo.store().has_language(lang)? {
        println!("Language '{}' is not cached.", lang);
        return Ok(());
    }

    repo.delete(lang)?;
    info!(lang, "deleted language");
    println!("Deleted {}.", lang);

    Ok(())
}
