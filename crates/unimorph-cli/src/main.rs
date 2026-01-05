//! UniMorph CLI - Command-line interface for UniMorph morphological data.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use unimorph_core::Repository;

#[derive(Parser)]
#[command(name = "unimorph")]
#[command(author, version, about = "Work with UniMorph morphological data", long_about = None)]
struct Cli {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download { lang, force } => cmd_download(&lang, force).await,
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

async fn cmd_download(lang: &str, force: bool) -> Result<()> {
    let mut repo = Repository::new().context("Failed to initialize repository")?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("valid template"),
    );
    pb.set_message(format!("Downloading {}...", lang));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let downloaded = if force {
        repo.refresh(lang).await.context("Download failed")?;
        true
    } else {
        repo.ensure(lang).await.context("Download failed")?
    };

    pb.finish_and_clear();

    if downloaded {
        let stats = repo.store().stats(lang)?.context("Failed to get stats")?;
        println!(
            "Downloaded {}: {} entries, {} lemmas, {} forms",
            lang, stats.total_entries, stats.unique_lemmas, stats.unique_forms
        );
    } else {
        println!("{} is already cached. Use --force to re-download.", lang);
    }

    Ok(())
}

fn cmd_list(cached: bool) -> Result<()> {
    let repo = Repository::new().context("Failed to initialize repository")?;

    if cached {
        let langs = repo.cached_languages()?;
        if langs.is_empty() {
            println!("No languages cached. Use 'unimorph download -l <lang>' to download.");
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
        // TODO: Fetch available languages from GitHub API
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

fn cmd_inflect(lang: &str, lemma: &str, features: Option<&str>, json: bool) -> Result<()> {
    let repo = Repository::new().context("Failed to initialize repository")?;

    if !repo.store().has_language(lang)? {
        anyhow::bail!(
            "Language '{}' not found. Download it first with: unimorph download -l {}",
            lang,
            lang
        );
    }

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

    if entries.is_empty() {
        if features.is_some() {
            println!(
                "No forms found for '{}' matching the feature pattern.",
                lemma
            );
        } else {
            println!("No forms found for '{}'.", lemma);
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

fn cmd_analyze(lang: &str, form: &str, json: bool) -> Result<()> {
    let repo = Repository::new().context("Failed to initialize repository")?;

    if !repo.store().has_language(lang)? {
        anyhow::bail!(
            "Language '{}' not found. Download it first with: unimorph download -l {}",
            lang,
            lang
        );
    }

    let entries = repo.store().analyze(lang, form)?;

    if entries.is_empty() {
        println!("No analyses found for '{}'.", form);
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

fn cmd_stats(lang: &str, json: bool) -> Result<()> {
    let repo = Repository::new().context("Failed to initialize repository")?;

    if !repo.store().has_language(lang)? {
        anyhow::bail!(
            "Language '{}' not found. Download it first with: unimorph download -l {}",
            lang,
            lang
        );
    }

    let stats = repo.store().stats(lang)?.context("Failed to get stats")?;

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

fn cmd_delete(lang: &str) -> Result<()> {
    let mut repo = Repository::new().context("Failed to initialize repository")?;

    if !repo.store().has_language(lang)? {
        println!("Language '{}' is not cached.", lang);
        return Ok(());
    }

    repo.delete(lang)?;
    println!("Deleted {}.", lang);

    Ok(())
}
