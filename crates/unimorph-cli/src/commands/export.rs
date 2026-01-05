//! Export command implementation.

use std::path::{Path, PathBuf};

use clap::ValueEnum;
use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::util::{create_repo, require_language, validate_lang_code};

/// Export format options.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Tsv,
    Jsonl,
    #[cfg(feature = "parquet")]
    Parquet,
}

#[instrument(skip_all, fields(lang))]
pub fn cmd_export(
    lang: &str,
    output: Option<PathBuf>,
    format: Option<ExportFormat>,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    // Determine format from flag or file extension
    let format = match (format, &output) {
        (Some(f), _) => f,
        (None, Some(path)) => match path.extension().and_then(|e| e.to_str()) {
            Some("tsv") => ExportFormat::Tsv,
            Some("jsonl") => ExportFormat::Jsonl,
            #[cfg(feature = "parquet")]
            Some("parquet") => ExportFormat::Parquet,
            _ => ExportFormat::Tsv, // default
        },
        (None, None) => ExportFormat::Tsv,
    };

    let output_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.tsv", lang)));

    let count = match format {
        ExportFormat::Tsv => repo.store().export_tsv(lang, &output_path)?,
        ExportFormat::Jsonl => repo.store().export_jsonl(lang, &output_path)?,
        #[cfg(feature = "parquet")]
        ExportFormat::Parquet => repo.store().export_parquet(lang, &output_path)?,
    };

    info!(
        lang,
        path = %output_path.display(),
        count,
        "export complete"
    );
    println!("Exported {} entries to {}", count, output_path.display());

    Ok(())
}
