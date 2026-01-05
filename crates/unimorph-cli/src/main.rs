//! UniMorph CLI - Command-line interface for UniMorph morphological data.

mod commands;
mod util;

use std::io;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use color_eyre::eyre::Result;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use commands::{
    ExportFormat, cmd_analyze, cmd_delete, cmd_download, cmd_export, cmd_inflect, cmd_list,
    cmd_search, cmd_stats,
};

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

#[derive(Subcommand)]
enum Commands {
    /// Download a language dataset
    Download {
        /// Language code (ISO 639-3, e.g., heb, vec, deu)
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
        Commands::List {
            cached,
            available,
            refresh,
            json,
        } => cmd_list(cached, available, refresh, json, cli.data_dir.as_deref()).await,
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
