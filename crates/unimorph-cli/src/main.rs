//! UniMorph CLI - Command-line interface for UniMorph morphological data.

use std::io::{self, IsTerminal};

use clap::{Parser, Subcommand};
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

    #[command(subcommand)]
    command: Commands,
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
        "starting unimorph CLI"
    );

    match cli.command {
        Commands::Download { lang, force } => cmd_download(&lang, force, cli.quiet).await,
        Commands::List { cached } => cmd_list(cached),
        Commands::Inflect {
            lang,
            lemma,
            features,
            json,
        } => cmd_inflect(&lang, &lemma, features.as_deref(), json),
        Commands::Analyze { lang, form, json } => cmd_analyze(&lang, &form, json),
        Commands::Stats { lang, json } => cmd_stats(&lang, json),
        Commands::Delete { lang } => cmd_delete(&lang),
    }
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
async fn cmd_download(lang: &str, force: bool, quiet: bool) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = Repository::new().context("Failed to initialize repository")?;

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

fn cmd_list(cached: bool) -> Result<()> {
    let repo = Repository::new().context("Failed to initialize repository")?;

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
fn cmd_inflect(lang: &str, lemma: &str, features: Option<&str>, json: bool) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = Repository::new().context("Failed to initialize repository")?;
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
fn cmd_analyze(lang: &str, form: &str, json: bool) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = Repository::new().context("Failed to initialize repository")?;
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
fn cmd_stats(lang: &str, json: bool) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = Repository::new().context("Failed to initialize repository")?;
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
fn cmd_delete(lang: &str) -> Result<()> {
    validate_lang_code(lang)?;

    let mut repo = Repository::new().context("Failed to initialize repository")?;

    if !repo.store().has_language(lang)? {
        println!("Language '{}' is not cached.", lang);
        return Ok(());
    }

    repo.delete(lang)?;
    info!(lang, "deleted language");
    println!("Deleted {}.", lang);

    Ok(())
}
