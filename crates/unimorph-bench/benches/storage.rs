//! Criterion benchmarks for storage backends.
//!
//! Run with: cargo bench --package unimorph-bench

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use unimorph_bench::{
    Entry, LangCode, Store, duckdb::DuckDbStore, parquet::ParquetStore, sqlite::SqliteStore,
};

fn sample_entries() -> Vec<Entry> {
    vec![
        Entry::parse_tsv_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
        Entry::parse_tsv_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
        Entry::parse_tsv_line("parlare\tparla\tV;IND;PRS;3;SG", 3).unwrap(),
        Entry::parse_tsv_line("parlare\tparliamo\tV;IND;PRS;1;PL", 4).unwrap(),
        Entry::parse_tsv_line("parlare\tparlate\tV;IND;PRS;2;PL", 5).unwrap(),
        Entry::parse_tsv_line("parlare\tparlano\tV;IND;PRS;3;PL", 6).unwrap(),
        Entry::parse_tsv_line("essere\tsono\tV;IND;PRS;1;SG", 7).unwrap(),
        Entry::parse_tsv_line("essere\tsei\tV;IND;PRS;2;SG", 8).unwrap(),
        Entry::parse_tsv_line("essere\tè\tV;IND;PRS;3;SG", 9).unwrap(),
        Entry::parse_tsv_line("avere\tho\tV;IND;PRS;1;SG", 10).unwrap(),
    ]
}

fn bench_lookup_by_lemma(c: &mut Criterion) {
    let entries = sample_entries();
    let lang = LangCode::new("ita").unwrap();

    let mut group = c.benchmark_group("lookup_by_lemma");

    // SQLite
    {
        let mut store = SqliteStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("sqlite", "parlare"), |b| {
            b.iter(|| store.lookup_by_lemma(black_box(&lang), black_box("parlare")))
        });
    }

    // DuckDB
    {
        let mut store = DuckDbStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("duckdb", "parlare"), |b| {
            b.iter(|| store.lookup_by_lemma(black_box(&lang), black_box("parlare")))
        });
    }

    // Parquet
    {
        let mut store = ParquetStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("parquet", "parlare"), |b| {
            b.iter(|| store.lookup_by_lemma(black_box(&lang), black_box("parlare")))
        });
    }

    group.finish();
}

fn bench_lookup_by_form(c: &mut Criterion) {
    let entries = sample_entries();
    let lang = LangCode::new("ita").unwrap();

    let mut group = c.benchmark_group("lookup_by_form");

    // SQLite
    {
        let mut store = SqliteStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("sqlite", "parlo"), |b| {
            b.iter(|| store.lookup_by_form(black_box(&lang), black_box("parlo")))
        });
    }

    // DuckDB
    {
        let mut store = DuckDbStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("duckdb", "parlo"), |b| {
            b.iter(|| store.lookup_by_form(black_box(&lang), black_box("parlo")))
        });
    }

    // Parquet
    {
        let mut store = ParquetStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("parquet", "parlo"), |b| {
            b.iter(|| store.lookup_by_form(black_box(&lang), black_box("parlo")))
        });
    }

    group.finish();
}

fn bench_stats(c: &mut Criterion) {
    let entries = sample_entries();
    let lang = LangCode::new("ita").unwrap();

    let mut group = c.benchmark_group("stats");

    // SQLite
    {
        let mut store = SqliteStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("sqlite", "ita"), |b| {
            b.iter(|| store.stats(black_box(&lang)))
        });
    }

    // DuckDB
    {
        let mut store = DuckDbStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("duckdb", "ita"), |b| {
            b.iter(|| store.stats(black_box(&lang)))
        });
    }

    // Parquet
    {
        let mut store = ParquetStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("parquet", "ita"), |b| {
            b.iter(|| store.stats(black_box(&lang)))
        });
    }

    group.finish();
}

fn bench_feature_search(c: &mut Criterion) {
    let entries = sample_entries();
    let lang = LangCode::new("ita").unwrap();

    let mut group = c.benchmark_group("feature_search");

    // SQLite
    {
        let mut store = SqliteStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("sqlite", "V;IND;PRS;*;SG"), |b| {
            b.iter(|| store.search_features(black_box(&lang), black_box("V;IND;PRS;*;SG")))
        });
    }

    // DuckDB
    {
        let mut store = DuckDbStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("duckdb", "V;IND;PRS;*;SG"), |b| {
            b.iter(|| store.search_features(black_box(&lang), black_box("V;IND;PRS;*;SG")))
        });
    }

    // Parquet
    {
        let mut store = ParquetStore::in_memory().unwrap();
        store.init(&lang, &entries).unwrap();

        group.bench_function(BenchmarkId::new("parquet", "V;IND;PRS;*;SG"), |b| {
            b.iter(|| store.search_features(black_box(&lang), black_box("V;IND;PRS;*;SG")))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_lookup_by_lemma,
    bench_lookup_by_form,
    bench_stats,
    bench_feature_search,
);

criterion_main!(benches);
