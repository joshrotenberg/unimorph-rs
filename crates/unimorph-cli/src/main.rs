//! UniMorph CLI - Command-line interface for UniMorph morphological data.

use std::io::{self, IsTerminal};
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use color_eyre::eyre::{Context, ContextCompat, Result, eyre};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info, instrument};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use unimorph_core::Repository;

#[derive(Parser)]
#[command(name = "unimorph")]
#[command(author, version, about = "Work with UniMorph morphological data", long_about = None)]
struct Cli {
    /// Enable verbose output (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Custom data directory (overrides UNIMORPH_DATA env var)
    #[arg(short, long, global = true, env = "UNIMORPH_DATA")]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

/// Export format options.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Tsv,
    Jsonl,
    #[cfg(feature = "parquet")]
    Parquet,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a language dataset
    Download {
        /// Language code (ISO 639-3, e.g., ita, fin, deu)
        #[arg(short, long)]
        lang: String,

        /// Force re-download even if cached
        #[arg(short, long)]
        force: bool,
    },

    /// List languages
    List {
        /// Show only cached (downloaded) languages
        #[arg(long)]
        cached: bool,
    },

    /// Look up all forms of a lemma (dictionary form)
    Inflect {
        /// Language code
        #[arg(short, long)]
        lang: String,

        /// Lemma to look up
        lemma: String,

        /// Filter by feature pattern (e.g., "V;IND;*;SG")
        #[arg(short, long)]
        features: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Analyze a surface form (reverse lookup)
    Analyze {
        /// Language code
        #[arg(short, long)]
        lang: String,

        /// Form to analyze
        form: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show dataset statistics
    Stats {
        /// Language code
        #[arg(short, long)]
        lang: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Delete a cached language dataset
    Delete {
        /// Language code
        #[arg(short, long)]
        lang: String,
    },

    /// Search entries with flexible filtering
    Search {
        /// Language code
        #[arg(short, long)]
        lang: String,

        /// Filter by lemma (supports SQL LIKE wildcards: % and _)
        #[arg(long)]
        lemma: Option<String>,

        /// Filter by surface form
        #[arg(long)]
        form: Option<String>,

        /// Filter by feature pattern (e.g., "V;IND;*;1;*")
        #[arg(short, long)]
        features: Option<String>,

        /// Filter by part of speech (e.g., V, N, ADJ)
        #[arg(long)]
        pos: Option<String>,

        /// Limit number of results
        #[arg(long, default_value = "100")]
        limit: usize,

        /// Skip first N results
        #[arg(long)]
        offset: Option<usize>,

        /// Just show count of matching entries
        #[arg(long)]
        count: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export a language dataset to file
    Export {
        /// Language code
        #[arg(short, long)]
        lang: String,

        /// Output file path (use - for stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (auto-detected from extension if not specified)
        #[arg(short, long)]
        format: Option<ExportFormat>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => EnvFilter::new("warn"),
        1 => EnvFilter::new("info,unimorph_core=debug,unimorph_cli=debug"),
        _ => EnvFilter::new("debug,unimorph_core=trace,unimorph_cli=trace"),
    };

    // Use RUST_LOG env var if set, otherwise use verbose flag
    let filter = EnvFilter::try_from_default_env().unwrap_or(filter);

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(verbose > 1))
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    debug!(
        verbose = cli.verbose,
        quiet = cli.quiet,
        data_dir = ?cli.data_dir,
        "starting unimorph CLI"
    );

    match cli.command {
        Commands::Download { lang, force } => {
            cmd_download(&lang, force, cli.quiet, cli.data_dir.as_deref()).await
        }
        Commands::List { cached } => cmd_list(cached, cli.data_dir.as_deref()),
        Commands::Inflect {
            lang,
            lemma,
            features,
            json,
        } => cmd_inflect(
            &lang,
            &lemma,
            features.as_deref(),
            json,
            cli.data_dir.as_deref(),
        ),
        Commands::Analyze { lang, form, json } => {
            cmd_analyze(&lang, &form, json, cli.data_dir.as_deref())
        }
        Commands::Stats { lang, json } => cmd_stats(&lang, json, cli.data_dir.as_deref()),
        Commands::Delete { lang } => cmd_delete(&lang, cli.data_dir.as_deref()),
        Commands::Search {
            lang,
            lemma,
            form,
            features,
            pos,
            limit,
            offset,
            count,
            json,
        } => cmd_search(
            &lang,
            lemma.as_deref(),
            form.as_deref(),
            features.as_deref(),
            pos.as_deref(),
            limit,
            offset,
            count,
            json,
            cli.data_dir.as_deref(),
        ),
        Commands::Export {
            lang,
            output,
            format,
        } => cmd_export(&lang, output, format, cli.data_dir.as_deref()),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "unimorph", &mut io::stdout());
            Ok(())
        }
    }
}

/// Create a repository, optionally with a custom data directory.
fn create_repo(data_dir: Option<&std::path::Path>) -> Result<Repository> {
    match data_dir {
        Some(path) => {
            debug!(path = %path.display(), "using custom data directory");
            Repository::with_cache_dir(path).context("Failed to initialize repository")
        }
        None => Repository::new().context("Failed to initialize repository"),
    }
}

/// Create a mutable repository, optionally with a custom data directory.
fn create_repo_mut(data_dir: Option<&std::path::Path>) -> Result<Repository> {
    create_repo(data_dir)
}

/// Validate a language code and provide helpful error messages.
fn validate_lang_code(lang: &str) -> Result<()> {
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
fn require_language(repo: &Repository, lang: &str) -> Result<()> {
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

#[instrument(skip_all, fields(lang, force))]
async fn cmd_download(
    lang: &str,
    force: bool,
    quiet: bool,
    data_dir: Option<&std::path::Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = create_repo_mut(data_dir)?;

    let is_terminal = io::stdout().is_terminal();
    let pb = if !quiet && is_terminal {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("valid template"),
        );
        pb.set_message(format!("Downloading {}...", lang));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let downloaded = if force {
        repo.refresh(lang)
            .await
            .context(format!("Failed to download '{}'", lang))?;
        true
    } else {
        repo.ensure(lang)
            .await
            .context(format!("Failed to download '{}'", lang))?
    };

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    if downloaded {
        let stats = repo
            .store()
            .stats(lang)?
            .context("Failed to retrieve stats after download")?;
        info!(
            lang,
            entries = stats.total_entries,
            lemmas = stats.unique_lemmas,
            forms = stats.unique_forms,
            "download complete"
        );
        if !quiet {
            println!(
                "Downloaded {}: {} entries, {} lemmas, {} forms",
                lang, stats.total_entries, stats.unique_lemmas, stats.unique_forms
            );
        }
    } else if !quiet {
        println!("{} is already cached. Use --force to re-download.", lang);
    }

    Ok(())
}

fn cmd_list(cached: bool, data_dir: Option<&std::path::Path>) -> Result<()> {
    let repo = create_repo(data_dir)?;

    if cached {
        let langs = repo.cached_languages()?;
        if langs.is_empty() {
            println!("No languages cached.");
            println!();
            println!("To download a language:");
            println!("  unimorph download -l <lang>");
            println!();
            println!("Examples:");
            println!("  unimorph download -l ita   # Italian");
            println!("  unimorph download -l deu   # German");
        } else {
            println!("Cached languages:");
            for lang in langs {
                let stats = repo.store().stats(lang.as_str())?;
                if let Some(stats) = stats {
                    println!("  {} ({} entries)", lang, stats.total_entries);
                } else {
                    println!("  {}", lang);
                }
            }
        }
    } else {
        println!("Available languages: https://github.com/unimorph");
        println!();
        println!("Common languages:");
        println!("  ita - Italian");
        println!("  deu - German");
        println!("  spa - Spanish");
        println!("  fra - French");
        println!("  fin - Finnish");
        println!("  rus - Russian");
        println!("  pol - Polish");
        println!("  tur - Turkish");
        println!();
        println!("Use 'unimorph list --cached' to see downloaded languages.");
    }

    Ok(())
}

#[instrument(skip_all, fields(lang, lemma))]
fn cmd_inflect(
    lang: &str,
    lemma: &str,
    features: Option<&str>,
    json: bool,
    data_dir: Option<&std::path::Path>,
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
    } else {
        println!("{:<20} {:<20} FEATURES", "LEMMA", "FORM");
        println!("{}", "-".repeat(60));
        for entry in &entries {
            println!("{:<20} {:<20} {}", entry.lemma, entry.form, entry.features);
        }
        println!();
        println!("{} form(s) found.", entries.len());
    }

    Ok(())
}

#[instrument(skip_all, fields(lang, form))]
fn cmd_analyze(
    lang: &str,
    form: &str,
    json: bool,
    data_dir: Option<&std::path::Path>,
) -> Result<()> {
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

#[instrument(skip_all, fields(lang))]
fn cmd_stats(lang: &str, json: bool, data_dir: Option<&std::path::Path>) -> Result<()> {
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

#[instrument(skip_all, fields(lang))]
fn cmd_delete(lang: &str, data_dir: Option<&std::path::Path>) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = create_repo_mut(data_dir)?;

    if !repo.store().has_language(lang)? {
        println!("Language '{}' is not cached.", lang);
        return Ok(());
    }

    repo.delete(lang)?;
    info!(lang, "deleted language");
    println!("Deleted {}.", lang);

    Ok(())
}

#[instrument(skip_all, fields(lang))]
#[allow(clippy::too_many_arguments)]
fn cmd_search(
    lang: &str,
    lemma: Option<&str>,
    form: Option<&str>,
    features: Option<&str>,
    pos: Option<&str>,
    limit: usize,
    offset: Option<usize>,
    count: bool,
    json: bool,
    data_dir: Option<&std::path::Path>,
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

#[instrument(skip_all, fields(lang))]
fn cmd_export(
    lang: &str,
    output: Option<PathBuf>,
    format: Option<ExportFormat>,
    data_dir: Option<&std::path::Path>,
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
