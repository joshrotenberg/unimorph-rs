//! Export command implementation.

use std::io::Write;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use color_eyre::eyre::Result;
#[cfg(feature = "parquet")]
use color_eyre::eyre::eyre;
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

    // Check if outputting to stdout
    let is_stdout = output.as_ref().is_some_and(|p| p.as_os_str() == "-");

    // Determine format from flag or file extension
    let format = match (format, &output) {
        (Some(f), _) => f,
        (None, Some(path)) if !is_stdout => match path.extension().and_then(|e| e.to_str()) {
            Some("tsv") => ExportFormat::Tsv,
            Some("jsonl") => ExportFormat::Jsonl,
            #[cfg(feature = "parquet")]
            Some("parquet") => ExportFormat::Parquet,
            _ => ExportFormat::Tsv, // default
        },
        _ => ExportFormat::Tsv,
    };

    if is_stdout {
        // Export to stdout
        #[cfg(feature = "parquet")]
        if matches!(format, ExportFormat::Parquet) {
            return Err(eyre!("Cannot export Parquet format to stdout"));
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        let count = match format {
            ExportFormat::Tsv => repo.store().export_tsv_to_writer(lang, &mut handle)?,
            ExportFormat::Jsonl => repo.store().export_jsonl_to_writer(lang, &mut handle)?,
            #[cfg(feature = "parquet")]
            ExportFormat::Parquet => unreachable!(),
        };

        handle.flush()?;

        // Print message to stderr so it doesn't pollute the data stream
        eprintln!("Exported {} entries to stdout", count);
        info!(lang, count, "export to stdout complete");
    } else {
        // Export to file
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
    }

    Ok(())
}
