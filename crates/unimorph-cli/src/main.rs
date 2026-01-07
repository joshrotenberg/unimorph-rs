//! UniMorph CLI - Command-line tool for working with UniMorph morphological data.

mod colors;
mod commands;
mod config;
mod util;

use std::io;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use color_eyre::eyre::Result;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use color_eyre::eyre::eyre;
use commands::{
    ExportFormat, cmd_analyze, cmd_config_init, cmd_config_path, cmd_config_show, cmd_delete,
    cmd_download, cmd_export, cmd_features, cmd_inflect, cmd_info, cmd_list, cmd_repair,
    cmd_sample, cmd_search, cmd_stats, cmd_update,
};
use config::Config;

/// Error message when no language is specified and no default is configured.
fn no_language_error() -> color_eyre::eyre::Report {
    eyre!(
        "No language specified.\n\n\
        Provide a language code as an argument, or set a default:\n\n\
        \x20 export UNIMORPH_LANG=heb\n\n\
        Or in ~/.config/unimorph/config.toml:\n\n\
        \x20 default_lang = \"heb\"\n\n\
        Run 'unimorph list --available' to see available languages."
    )
}

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n",
    "https://github.com/joshrotenberg/unimorph-rs"
);

#[derive(Parser)]
#[command(name = "unimorph")]
#[command(author, version, long_version = LONG_VERSION, about = "Work with UniMorph morphological data", long_about = None)]
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

#[derive(Subcommand)]
enum Commands {
    /// Download a language dataset
    #[command(visible_alias = "dl")]
    Download {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        lang: Option<String>,

        /// Force re-download even if cached
        #[arg(short, long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List languages
    #[command(visible_alias = "ls")]
    List {
        /// Show only cached (downloaded) languages
        #[arg(long)]
        cached: bool,

        /// Fetch available languages from GitHub
        #[arg(long)]
        available: bool,

        /// Refresh the cached list of available languages
        #[arg(long)]
        refresh: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Look up all forms of a lemma (dictionary form)
    #[command(visible_alias = "i")]
    Inflect {
        /// Lemma to look up
        lemma: String,

        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// Filter by feature pattern (e.g., "V;IND;*;SG")
        #[arg(short, long)]
        features: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output as TSV (tab-separated, no headers) for piping
        #[arg(long)]
        tsv: bool,
    },

    /// Analyze a surface form (reverse lookup)
    #[command(visible_alias = "a")]
    Analyze {
        /// Form to analyze
        form: String,

        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output as TSV (tab-separated, no headers) for piping
        #[arg(long)]
        tsv: bool,
    },

    /// Show dataset statistics
    #[command(visible_alias = "st")]
    Stats {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        lang: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Delete a cached language dataset
    #[command(visible_alias = "rm")]
    Delete {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        lang: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search entries with flexible filtering
    #[command(visible_alias = "s")]
    Search {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// Filter by lemma (supports SQL LIKE wildcards: % and _)
        #[arg(long)]
        lemma: Option<String>,

        /// Filter by surface form (supports SQL LIKE wildcards: % and _)
        #[arg(long)]
        form: Option<String>,

        /// Filter by feature pattern (e.g., "V;IND;*;1;*")
        #[arg(short, long)]
        features: Option<String>,

        /// Filter by features contained (comma-separated, position-independent)
        /// Example: --contains PL,MASC finds entries with both PL and MASC
        #[arg(short, long, value_delimiter = ',')]
        contains: Option<Vec<String>>,

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

        /// Output as TSV (tab-separated, no headers) for piping
        #[arg(long)]
        tsv: bool,
    },

    /// Export a language dataset to file
    #[command(visible_alias = "x")]
    Export {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// Output file path (use - for stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (auto-detected from extension if not specified)
        #[arg(short = 'F', long)]
        format: Option<ExportFormat>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Show detailed info about a cached language
    #[command(visible_alias = "in")]
    Info {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        lang: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Update cached language datasets
    #[command(visible_alias = "up")]
    Update {
        /// Language code (omit with --all to update all)
        lang: Option<String>,

        /// Update all cached languages
        #[arg(long)]
        all: bool,

        /// Check for updates without downloading
        #[arg(long)]
        check: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Repair or reset the local data store
    Repair {
        /// Clear cached API responses
        #[arg(long)]
        clear_cache: bool,

        /// Clear all downloaded datasets (will need to re-download)
        #[arg(long)]
        clear_data: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Explore morphological features in a language
    #[command(visible_alias = "f")]
    Features {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// List all unique feature values
        #[arg(long)]
        list: bool,

        /// Show feature value counts (histogram)
        #[arg(long)]
        stats: bool,

        /// Search for entries containing a specific feature
        #[arg(long)]
        search: Option<String>,

        /// Show values at a specific position (0-indexed)
        #[arg(long)]
        position: Option<usize>,

        /// Limit number of results
        #[arg(long, default_value = "50")]
        limit: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Randomly sample entries from a language dataset
    #[command(visible_alias = "rand")]
    Sample {
        /// Number of entries to sample
        n: usize,

        /// Language code (ISO 639-3, e.g., heb, vec, deu)
        /// Uses UNIMORPH_LANG env var or config default if not specified
        #[arg(short, long)]
        lang: Option<String>,

        /// Seed for reproducible sampling
        #[arg(short, long)]
        seed: Option<u64>,

        /// Sample complete paradigms (all forms of selected lemmas)
        /// instead of random individual entries
        #[arg(long)]
        by_lemma: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output as TSV (tab-separated, no headers) for piping
        #[arg(long)]
        tsv: bool,
    },

    /// Manage configuration
    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Initialize a new config file with example content
    Init {
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show the config file path
    Path {
        /// Output as JSON
        #[arg(long)]
        json: bool,
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

/// Resolve the effective data directory.
///
/// Priority order:
/// 1. CLI --data-dir flag
/// 2. UNIMORPH_DATA env var (handled by clap)
/// 3. Config file data_dir
/// 4. Default (~/.cache/unimorph)
fn resolve_data_dir(cli_data_dir: Option<PathBuf>, config: &Config) -> Option<PathBuf> {
    // CLI flag (including env var via clap) takes precedence
    if cli_data_dir.is_some() {
        return cli_data_dir;
    }
    // Fall back to config file
    config.data_dir.clone()
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    // Load configuration file
    let config = Config::load();
    debug!(config = ?config, "loaded configuration");

    // Resolve data directory (CLI > env > config > default)
    let data_dir = resolve_data_dir(cli.data_dir, &config);

    debug!(
        verbose = cli.verbose,
        quiet = cli.quiet,
        data_dir = ?data_dir,
        "starting unimorph CLI"
    );

    match cli.command {
        Commands::Download { lang, force, json } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_download(&lang, force, json, cli.quiet, data_dir.as_deref()).await
        }
        Commands::List {
            cached,
            available,
            refresh,
            json,
        } => cmd_list(cached, available, refresh, json, data_dir.as_deref()).await,
        Commands::Inflect {
            lemma,
            lang,
            features,
            json,
            tsv,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_inflect(
                &lang,
                &lemma,
                features.as_deref(),
                json,
                tsv,
                data_dir.as_deref(),
            )
        }
        Commands::Analyze {
            form,
            lang,
            json,
            tsv,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_analyze(&lang, &form, json, tsv, data_dir.as_deref())
        }
        Commands::Stats { lang, json } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_stats(&lang, json, data_dir.as_deref())
        }
        Commands::Delete { lang, json } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_delete(&lang, json, data_dir.as_deref())
        }
        Commands::Search {
            lang,
            lemma,
            form,
            features,
            contains,
            pos,
            limit,
            offset,
            count,
            json,
            tsv,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_search(
                &lang,
                lemma.as_deref(),
                form.as_deref(),
                features.as_deref(),
                contains,
                pos.as_deref(),
                limit,
                offset,
                count,
                json,
                tsv,
                data_dir.as_deref(),
            )
        }
        Commands::Export {
            lang,
            output,
            format,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_export(&lang, output, format, data_dir.as_deref())
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "unimorph", &mut io::stdout());
            Ok(())
        }
        Commands::Info { lang, json } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_info(&lang, json, data_dir.as_deref()).await
        }
        Commands::Update {
            lang,
            all,
            check,
            json,
        } => {
            // Update command: lang is optional if --all is used
            let lang = if all {
                None
            } else {
                Some(
                    config
                        .effective_lang(lang.as_deref())
                        .ok_or_else(no_language_error)?,
                )
            };
            cmd_update(lang.as_deref(), all, check, json, data_dir.as_deref()).await
        }
        Commands::Repair {
            clear_cache,
            clear_data,
            json,
        } => cmd_repair(clear_cache, clear_data, json, data_dir.as_deref()),
        Commands::Features {
            lang,
            list,
            stats,
            search,
            position,
            limit,
            json,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_features(
                &lang,
                list,
                stats,
                search.as_deref(),
                position,
                limit,
                json,
                data_dir.as_deref(),
            )
        }
        Commands::Sample {
            n,
            lang,
            seed,
            by_lemma,
            json,
            tsv,
        } => {
            let lang = config
                .effective_lang(lang.as_deref())
                .ok_or_else(no_language_error)?;
            cmd_sample(&lang, n, seed, by_lemma, json, tsv, data_dir.as_deref())
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { json } => cmd_config_show(json),
            ConfigAction::Init { force, json } => cmd_config_init(force, json),
            ConfigAction::Path { json } => cmd_config_path(json),
        },
    }
}
