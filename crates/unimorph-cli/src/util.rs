//! Common utilities for CLI commands.

use std::path::Path;

use color_eyre::eyre::{Context, Result, eyre};
use tracing::debug;
use unimorph_core::Repository;

/// Create a repository, optionally with a custom data directory.
pub fn create_repo(data_dir: Option<&Path>) -> Result<Repository> {
    match data_dir {
        Some(path) => {
            debug!(path = %path.display(), "using custom data directory");
            Repository::with_cache_dir(path).context("Failed to initialize repository")
        }
        None => Repository::new().context("Failed to initialize repository"),
    }
}

/// Validate a language code and provide helpful error messages.
pub fn validate_lang_code(lang: &str) -> Result<()> {
    if lang.len() != 3 {
        return Err(eyre!(
            "Invalid language code: '{}'\n\n\
            Language codes must be exactly 3 lowercase letters (ISO 639-3).\n\
            Examples: ita (Italian), deu (German), fin (Finnish)\n\n\
            See https://iso639-3.sil.org/code_tables/639/data for the full list.",
            lang
        ));
    }

    if !lang.chars().all(|c| c.is_ascii_lowercase()) {
        let suggestion = lang.to_lowercase();
        return Err(eyre!(
            "Invalid language code: '{}'\n\n\
            Language codes must be lowercase. Did you mean '{}'?",
            lang,
            suggestion
        ));
    }

    Ok(())
}

/// Check if a language is downloaded, with a helpful error if not.
pub fn require_language(repo: &Repository, lang: &str) -> Result<()> {
    if !repo.store().has_language(lang)? {
        return Err(eyre!(
            "Language '{}' is not downloaded.\n\n\
            To download it, run:\n\
            \n    unimorph download -l {}\n\n\
            To see what's cached:\n\
            \n    unimorph list --cached",
            lang,
            lang
        ));
    }
    Ok(())
}
