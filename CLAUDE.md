# unimorph-rs

A complete Rust toolkit for working with UniMorph morphological data.

## Project Status

**Current phase**: Core library development (post-benchmarking)

### Completed
- Storage backend benchmarks (SQLite, DuckDB, Parquet)
- Decision: SQLite for runtime, Parquet for export only
- Core types validated: `Entry`, `FeatureBundle`, `LangCode`

### In Progress
- `unimorph-core` crate development

## Architecture Decisions

### Storage Strategy

**SQLite: Primary runtime store**
- 10-100x faster on point lookups (the main use case)
- B-tree indexes on `(lang, lemma)` and `(lang, form)`
- Single file at `~/.cache/unimorph/datasets.db`
- Stats pre-computed at import time in `meta` table
- Mature, single-file, zero-config

**Parquet: Export format only**
- 50-200x smaller than SQLite
- Generate lazily on `export` command
- For users who want Polars/DuckDB/Pandas/ML pipelines

**DuckDB: Not needed in core**
- Users can export to Parquet and query directly
- No need to maintain second backend

**TSV: Ephemeral download artifact**
- Fetch from upstream, stream into SQLite, discard

**Feature search: Future work (not v1)**
- Currently slow everywhere (1-6 ops/sec)
- FTS5 on tokenized features if needed later

### SQLite Schema

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;      -- 64MB
PRAGMA mmap_size = 268435456;    -- 256MB

CREATE TABLE entries (
    id INTEGER PRIMARY KEY,
    lang TEXT NOT NULL,
    lemma TEXT NOT NULL,
    form TEXT NOT NULL,
    features TEXT NOT NULL
);

CREATE INDEX idx_lang_lemma ON entries(lang, lemma);
CREATE INDEX idx_lang_form ON entries(lang, form);

CREATE TABLE meta (
    lang TEXT PRIMARY KEY,
    entry_count INTEGER,
    unique_lemmas INTEGER,
    unique_forms INTEGER,
    unique_features INTEGER,
    imported_at TEXT,
    source_url TEXT
);
```

## Target Users

### 1. Language learners / educators
- Conjugation practice apps
- Flashcard generators (Anki)
- Grammar reference lookups
- Need: simple queries, fast responses, offline support

### 2. NLP researchers / computational linguists
- Training data for morphological inflection models (SIGMORPHON)
- Linguistic typology studies
- Cross-lingual analysis
- Need: bulk export, DataFrame integration, feature filtering

### 3. NLP engineers / ML practitioners
- Data augmentation for NLU models
- Lemmatization/stemming lookup tables
- Morphological analysis in pipelines
- Need: library integration, batch APIs, streaming

### 4. Lexicographers / dictionary builders
- Generating inflection tables
- Validating coverage
- Need: completeness checks, paradigm views

## Crate Structure

```
unimorph-rs/
├── crates/
│   ├── unimorph-core/       # Types, storage, query engine (v1)
│   │   ├── types.rs         # Entry, FeatureBundle, LangCode
│   │   ├── store.rs         # SQLite backend
│   │   ├── repository.rs    # Download, cache management
│   │   └── query.rs         # Query builders, result iterators
│   │
│   ├── unimorph-cli/        # CLI binary (v1)
│   │
│   ├── unimorph-bench/      # Benchmarks (done)
│   │
│   ├── unimorph-python/     # PyO3 bindings (v2)
│   │                        # Returns Polars DataFrames natively
│   │
│   └── unimorph-server/     # REST API (v2)
│                            # Axum, OpenAPI spec
```

## Interface Priorities

| Interface | Primary Users | Version |
|-----------|---------------|---------|
| CLI | All | v1 |
| Rust library | NLP engineers | v1 |
| Python bindings (PyO3) | Researchers, ML | v2 |
| REST API | App developers | v2 |
| Polars integration | Data scientists | v2 |
| WASM | Browser tools | v3 |
| GraphQL | Frontend devs | v3 |

## API Design Principles

### Iterator-based results
Large queries shouldn't OOM. Core returns iterators, consumers collect or stream.

```rust
// Streams results, doesn't load all into memory
for entry in store.query().lang("fin").lemma("puhua").execute()? {
    println!("{}", entry.form);
}
```

### Query builder pattern
```rust
store.query()
    .lang("ita")
    .lemma("parlare")
    .features_contain("IND")
    .limit(100)
    .execute()?
```

### Batch operations for ML pipelines
```rust
// Lookup many lemmas at once
store.batch_inflect(&["parlare", "essere", "avere"])?

// Analyze a text's tokens
store.batch_analyze(&["parlo", "sono", "fatto"])?
```

### Export as first-class operation
```rust
store.export_parquet(lang, path)?
store.export_tsv(lang, writer)?
```

CLI:
```bash
unimorph export ita --format parquet -o italian.parquet
```

## Future Features (post-v1)

- **Paradigm tables**: Structured conjugation/declension grids
- **Diff/changelog**: Compare UniMorph versions
- **Coverage reports**: Compare against Wiktionary/other sources
- **Random sampling**: `store.sample(lang, n)` for ML splits
- **GitHub integration**: CLI to facilitate PRs/issues for language repos

## Benchmark Results Summary

Tested on: ita (510k), fin (2.7M), deu (519k), spa (1.2M), ces (134k), pol (201k)

| Backend | Lookup ops/sec | Compression |
|---------|---------------|-------------|
| SQLite | 15k-215k | 1x (baseline) |
| DuckDB | 500-2500 | ~0.7x |
| Parquet | 100-600 | 0.005-0.02x |

SQLite wins decisively for point lookups. Parquet wins for storage/distribution.

Full results in `crates/unimorph-bench/results/RESULTS.md`.

## Development Commands

```bash
# Format, lint, test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all-features
cargo test --test '*' --all-features

# Run benchmarks
cargo run --release --bin setup -- --langs ita,fin
cargo run --release --bin bench -- --lang ita
```
