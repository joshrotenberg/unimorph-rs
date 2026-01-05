//! Benchmark binary for comparing storage backends.
//!
//! This is a simple benchmark runner. For proper statistical analysis,
//! use the criterion benchmarks in benches/storage.rs.
//!
//! Usage:
//!   cargo run --bin bench
//!   cargo run --bin bench -- --lang ita --iterations 1000
//!   cargo run --bin bench -- --lang ita --json >> results/benchmarks.jsonl

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;

use unimorph_bench::{
    LangCode, Result, Store, duckdb::DuckDbStore, parquet::ParquetStore, sqlite::SqliteStore,
};

struct BenchConfig {
    lang: LangCode,
    iterations: usize,
    cache_dir: PathBuf,
    json_output: bool,
}

impl BenchConfig {
    fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        let mut lang: LangCode = "ita".parse()?;
        let mut iterations = 1000;
        let mut cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("unimorph-bench");
        let mut json_output = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--lang" | "-l" => {
                    i += 1;
                    if i < args.len() {
                        lang = args[i].parse()?;
                    }
                }
                "--iterations" | "-n" => {
                    i += 1;
                    if i < args.len() {
                        iterations = args[i].parse().unwrap_or(1000);
                    }
                }
                "--cache-dir" | "-c" => {
                    i += 1;
                    if i < args.len() {
                        cache_dir = PathBuf::from(&args[i]);
                    }
                }
                "--json" | "-j" => {
                    json_output = true;
                }
                "--help" | "-h" => {
                    println!("Usage: bench [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!("  -l, --lang <CODE>       Language code (default: ita)");
                    println!("  -n, --iterations <N>    Number of iterations (default: 1000)");
                    println!("  -c, --cache-dir <PATH>  Cache directory");
                    println!("  -j, --json              Output results as JSON lines");
                    println!("  -h, --help              Show this help");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[i]);
                    std::process::exit(1);
                }
            }
            i += 1;
        }

        Ok(Self {
            lang,
            iterations,
            cache_dir,
            json_output,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct BenchResult {
    name: String,
    total_time_ms: f64,
    iterations: usize,
    avg_micros: f64,
    ops_per_sec: f64,
}

impl BenchResult {
    fn new(name: String, total_time: Duration, iterations: usize) -> Self {
        let avg_micros = total_time.as_micros() as f64 / iterations as f64;
        let ops_per_sec = iterations as f64 / total_time.as_secs_f64();
        Self {
            name,
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            iterations,
            avg_micros,
            ops_per_sec,
        }
    }
}

#[derive(Debug, Serialize)]
struct BenchRun {
    timestamp: String,
    lang: String,
    entries: usize,
    backend: String,
    db_size_bytes: u64,
    results: Vec<BenchResult>,
}

/// Get sample lemmas for a language.
/// These are common verbs that should exist in each dataset.
fn get_sample_lemmas(lang: &LangCode) -> Vec<&'static str> {
    match lang.as_str() {
        "ita" => vec!["parlare", "essere", "avere", "fare", "andare"],
        "fin" => vec!["puhua", "olla", "tehdä", "mennä", "tulla"],
        "deu" => vec!["sprechen", "sein", "haben", "machen", "gehen"],
        "spa" => vec!["hablar", "ser", "tener", "hacer", "ir"],
        "fra" => vec!["parler", "être", "avoir", "faire", "aller"],
        "rus" => vec!["говорить", "быть", "иметь", "делать", "идти"],
        "pol" => vec!["mówić", "być", "mieć", "robić", "iść"],
        "tur" => vec!["konuşmak", "olmak", "yapmak", "gitmek", "gelmek"],
        _ => vec!["be", "have", "do", "go", "say"], // Fallback
    }
}

/// Get sample forms for a language.
fn get_sample_forms(lang: &LangCode) -> Vec<&'static str> {
    match lang.as_str() {
        "ita" => vec!["parlo", "sono", "ho", "faccio", "vado"],
        "fin" => vec!["puhun", "olen", "teen", "menen", "tulen"],
        "deu" => vec!["spreche", "bin", "habe", "mache", "gehe"],
        "spa" => vec!["hablo", "soy", "tengo", "hago", "voy"],
        "fra" => vec!["parle", "suis", "ai", "fais", "vais"],
        "rus" => vec!["говорю", "есть", "имею", "делаю", "иду"],
        "pol" => vec!["mówię", "jestem", "mam", "robię", "idę"],
        "tur" => vec!["konuşurum", "olurum", "yaparım", "giderim", "gelirim"],
        _ => vec!["am", "have", "do", "go", "say"],
    }
}

/// Get a feature pattern appropriate for the language.
fn get_feature_pattern(lang: &LangCode) -> &'static str {
    match lang.as_str() {
        // Most languages have indicative present verbs
        "ita" | "spa" | "fra" | "deu" | "pol" | "rus" => "V;IND;PRS;*;SG",
        // Finnish uses slightly different features
        "fin" => "V;PRS;*;SG",
        // Turkish agglutinative
        "tur" => "V;PRS;*",
        _ => "V;*;SG",
    }
}

fn bench_lookup_by_lemma<S: Store>(
    store: &S,
    lang: &LangCode,
    lemmas: &[&str],
    iterations: usize,
) -> BenchResult {
    let start = Instant::now();

    for i in 0..iterations {
        let lemma = lemmas[i % lemmas.len()];
        let _ = store.lookup_by_lemma(lang, lemma);
    }

    BenchResult::new("lookup_by_lemma".to_string(), start.elapsed(), iterations)
}

fn bench_lookup_by_form<S: Store>(
    store: &S,
    lang: &LangCode,
    forms: &[&str],
    iterations: usize,
) -> BenchResult {
    let start = Instant::now();

    for i in 0..iterations {
        let form = forms[i % forms.len()];
        let _ = store.lookup_by_form(lang, form);
    }

    BenchResult::new("lookup_by_form".to_string(), start.elapsed(), iterations)
}

fn bench_stats<S: Store>(store: &S, lang: &LangCode, iterations: usize) -> BenchResult {
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = store.stats(lang);
    }

    BenchResult::new("stats".to_string(), start.elapsed(), iterations)
}

fn bench_feature_search<S: Store>(
    store: &S,
    lang: &LangCode,
    pattern: &str,
    iterations: usize,
) -> BenchResult {
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = store.search_features(lang, pattern);
    }

    BenchResult::new(
        format!("search_features({})", pattern),
        start.elapsed(),
        iterations,
    )
}

fn print_results(backend: &str, results: &[BenchResult]) {
    println!("\n{}", backend);
    println!("{}", "=".repeat(backend.len()));
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "Benchmark", "Total (ms)", "Avg (us)", "Ops/sec"
    );
    println!("{}", "-".repeat(70));

    for r in results {
        println!(
            "{:<30} {:>12.2} {:>12.2} {:>12.0}",
            r.name, r.total_time_ms, r.avg_micros, r.ops_per_sec
        );
    }
}

fn print_json(run: &BenchRun) {
    if let Ok(json) = serde_json::to_string(run) {
        println!("{}", json);
    }
}

fn get_entry_count<S: Store>(store: &S, lang: &LangCode) -> usize {
    store
        .stats(lang)
        .map(|s| s.total_entries)
        .unwrap_or_default()
}

fn main() -> Result<()> {
    let config = BenchConfig::parse()?;

    let timestamp = chrono_lite_timestamp();

    if !config.json_output {
        println!("UniMorph Storage Benchmarks");
        println!("===========================");
        println!("Language: {}", config.lang);
        println!("Iterations: {}", config.iterations);
        println!("Cache dir: {}", config.cache_dir.display());
    }

    let lemmas = get_sample_lemmas(&config.lang);
    let forms = get_sample_forms(&config.lang);
    let pattern = get_feature_pattern(&config.lang);

    // SQLite
    let sqlite_path = config.cache_dir.join("unimorph.sqlite");
    if sqlite_path.exists() {
        let store = SqliteStore::open(&sqlite_path)?;
        let entry_count = get_entry_count(&store, &config.lang);
        let db_size = std::fs::metadata(&sqlite_path)?.len();

        let results = vec![
            bench_lookup_by_lemma(&store, &config.lang, &lemmas, config.iterations),
            bench_lookup_by_form(&store, &config.lang, &forms, config.iterations),
            bench_stats(&store, &config.lang, config.iterations.min(100)),
            bench_feature_search(&store, &config.lang, pattern, config.iterations.min(10)),
        ];

        if config.json_output {
            print_json(&BenchRun {
                timestamp: timestamp.clone(),
                lang: config.lang.as_str().to_string(),
                entries: entry_count,
                backend: "sqlite".to_string(),
                db_size_bytes: db_size,
                results,
            });
        } else {
            print_results("SQLite", &results);
        }
    } else if !config.json_output {
        println!("\nSQLite database not found. Run setup first.");
    }

    // DuckDB
    let duckdb_path = config.cache_dir.join("unimorph.duckdb");
    if duckdb_path.exists() {
        let store = DuckDbStore::open(&duckdb_path)?;
        let entry_count = get_entry_count(&store, &config.lang);
        let db_size = std::fs::metadata(&duckdb_path)?.len();

        let results = vec![
            bench_lookup_by_lemma(&store, &config.lang, &lemmas, config.iterations),
            bench_lookup_by_form(&store, &config.lang, &forms, config.iterations),
            bench_stats(&store, &config.lang, config.iterations.min(100)),
            bench_feature_search(&store, &config.lang, pattern, config.iterations.min(10)),
        ];

        if config.json_output {
            print_json(&BenchRun {
                timestamp: timestamp.clone(),
                lang: config.lang.as_str().to_string(),
                entries: entry_count,
                backend: "duckdb".to_string(),
                db_size_bytes: db_size,
                results,
            });
        } else {
            print_results("DuckDB", &results);
        }
    } else if !config.json_output {
        println!("\nDuckDB database not found. Run setup first.");
    }

    // Parquet
    let parquet_dir = config.cache_dir.join("parquet");
    let parquet_file = parquet_dir.join(format!("{}.parquet", config.lang.as_str()));
    if parquet_file.exists() {
        let store = ParquetStore::new(parquet_dir)?;
        let entry_count = get_entry_count(&store, &config.lang);
        let db_size = std::fs::metadata(&parquet_file)?.len();

        let results = vec![
            bench_lookup_by_lemma(&store, &config.lang, &lemmas, config.iterations),
            bench_lookup_by_form(&store, &config.lang, &forms, config.iterations),
            bench_stats(&store, &config.lang, config.iterations.min(100)),
            bench_feature_search(&store, &config.lang, pattern, config.iterations.min(10)),
        ];

        if config.json_output {
            print_json(&BenchRun {
                timestamp,
                lang: config.lang.as_str().to_string(),
                entries: entry_count,
                backend: "parquet".to_string(),
                db_size_bytes: db_size,
                results,
            });
        } else {
            print_results("Parquet + Polars", &results);
        }
    } else if !config.json_output {
        println!("\nParquet files not found. Run setup first.");
    }

    if !config.json_output {
        println!("\nDone!");
    }

    Ok(())
}

/// Simple timestamp without pulling in chrono.
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
