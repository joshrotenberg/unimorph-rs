# unimorph-rs

[![Documentation](https://img.shields.io/badge/docs-mdBook-blue)](https://joshrotenberg.github.io/unimorph-rs/)
[![Crates.io](https://img.shields.io/crates/v/unimorph-core)](https://crates.io/crates/unimorph-core)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

A Rust toolkit for working with [UniMorph](https://unimorph.github.io/) morphological data.

**[Documentation](https://joshrotenberg.github.io/unimorph-rs/)** | **[Crates.io](https://crates.io/crates/unimorph-core)** | **[GitHub](https://github.com/joshrotenberg/unimorph-rs)**

## What is UniMorph?

UniMorph is a collaborative project out of Johns Hopkins CLSP that provides morphological paradigm data for 169+ languages in a unified annotation format. It's the de facto standard dataset for morphological inflection research in NLP, used extensively in the annual SIGMORPHON shared tasks.

Each entry is a triple:
```
lemma       form        features
parlare     parlo       V;IND;PRS;1;SG
parlare     parlato     V.PTCP;PST
essere      sono        V;IND;PRS;1;SG
```

The schema includes 23 dimensions of meaning (tense, aspect, mood, person, number, case, gender, etc.) with 212+ features.

## Why This Project?

The existing Python tooling (`pip install unimorph`, v0.0.4) is minimal:
- Shells out to `git clone` for downloads
- Loads entire datasets into pandas for every query
- No caching, no indexing, no streaming
- Limited query capabilities (exact match only)

A Rust rewrite offers:
- **Performance**: Polars-based columnar queries, memory-mapped files, proper indexing
- **Better CLI UX**: Rich subcommands, fuzzy matching, feature pattern queries, multiple output formats
- **Library + CLI**: Usable as both a Rust crate and command-line tool
- **Schema validation**: Validate feature bundles against the official UniMorph schema

## Project Structure

```
unimorph-rs/
├── crates/
│   ├── unimorph-core/      # Data structures, parsing, Polars-based queries
│   ├── unimorph-schema/    # Schema validation, canonicalization
│   └── unimorph-cli/       # CLI binary
├── Cargo.toml              # Workspace manifest
└── README.md
```

## Core Types

### `LangCode`
ISO 639-3 language code (3 lowercase ASCII letters): `ita`, `eng`, `deu`, etc.

### `Entry`
A single morphological entry:
```rust
pub struct Entry {
    pub lemma: String,      // Dictionary form
    pub form: String,       // Inflected surface form  
    pub features: FeatureBundle,
}
```

### `FeatureBundle`
Semicolon-separated morphological features:
```rust
pub struct FeatureBundle {
    raw: String,            // Original string for round-tripping
    features: Vec<String>,  // Parsed individual features
}

// Supports pattern matching:
bundle.matches_pattern("V;IND;*;1;*")  // Any 1st person indicative verb
```

### `Dataset`
A loaded UniMorph dataset backed by Polars DataFrame:
```rust
impl Dataset {
    fn from_tsv(path: &Path, lang: LangCode) -> Result<Self>;
    fn inflect(&self, lemma: &str) -> Result<DataFrame>;
    fn analyze(&self, form: &str) -> Result<DataFrame>;
    fn search_features(&self, pattern: &str) -> Result<DataFrame>;
    fn stats(&self) -> Result<DatasetStats>;
}
```

### `Repository`
Handles downloading and caching datasets:
```rust
impl Repository {
    fn new() -> Result<Self>;  // Uses ~/.cache/unimorph
    async fn ensure_dataset(&self, lang: &LangCode) -> Result<PathBuf>;
    async fn list_available(&self) -> Result<Vec<LangCode>>;
    async fn list_cached(&self) -> Result<Vec<LangCode>>;
}
```

## Shell Completions

Enable tab completion for your shell:

### Bash

```bash
# Add to ~/.bashrc
eval "$(unimorph completions bash)"

# Or generate to a file (recommended for faster shell startup)
unimorph completions bash > ~/.local/share/bash-completion/completions/unimorph
```

### Zsh

```bash
# Add to ~/.zshrc (before compinit)
eval "$(unimorph completions zsh)"

# Or generate to a file
unimorph completions zsh > ~/.zfunc/_unimorph
# Then add to ~/.zshrc: fpath+=~/.zfunc
```

### Fish

```bash
unimorph completions fish > ~/.config/fish/completions/unimorph.fish
```

### PowerShell

```powershell
# Add to your PowerShell profile
unimorph completions powershell | Out-String | Invoke-Expression
```

## CLI Design

```bash
# Download datasets
unimorph download -l ita
unimorph download -l ita --force  # Re-download

# List languages
unimorph list              # Available (from GitHub API)
unimorph list --cached     # Downloaded locally

# Query paradigms
unimorph inflect -l ita parlare
unimorph inflect -l ita parlare --features "V;IND;PRS;*"
unimorph inflect -l ita parlare --json

# Analyze surface forms
unimorph analyze -l ita parlo
unimorph analyze -l ita "sono"  # Might have multiple analyses

# Dataset stats
unimorph stats -l ita

# Citation info
unimorph cite
```

Example output:
```
$ unimorph inflect -l ita parlare

LEMMA     FORM        FEATURES
parlare   parlo       V;IND;PRS;1;SG
parlare   parli       V;IND;PRS;2;SG
parlare   parla       V;IND;PRS;3;SG
parlare   parliamo    V;IND;PRS;1;PL
parlare   parlate     V;IND;PRS;2;PL
parlare   parlano     V;IND;PRS;3;PL
...
```

## Implementation Phases

### Phase 1: Core Library + Basic CLI ✓ (sketched)
- [x] Core types: `Entry`, `FeatureBundle`, `LangCode`
- [x] Polars-based `Dataset` with basic queries
- [x] `Repository` for async downloads with caching
- [x] CLI with download, list, inflect, analyze, stats commands

### Phase 2: Schema Validation
- [ ] Port `tags.yaml` schema definition from um-canonicalize
- [ ] Validate feature bundles against schema
- [ ] Canonicalization (standard feature ordering)
- [ ] CLI `validate` command

### Phase 3: Advanced Features
- [ ] Persistent disk indexes (avoid re-parsing large datasets)
- [ ] Fuzzy matching for lemma/form queries
- [ ] Feature pattern queries with full wildcard support
- [ ] Export to other formats (CoNLL-U, JSON-L)
- [ ] Batch processing from stdin

### Phase 4: Ecosystem
- [ ] Python bindings via PyO3 (return Polars DataFrames)
- [ ] WASM build for browser use
- [ ] Language metadata (family, typology, source info)

## Key Dependencies

```toml
# Core
polars = { version = "0.46", features = ["lazy", "csv"] }
thiserror = "2"
anyhow = "1"
serde = { version = "1", features = ["derive"] }

# Async/IO
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }

# CLI
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"
console = "0.15"
tracing = "0.1"
```

## UniMorph Data Notes

### Data Format
- Tab-separated, no header, 3 columns: lemma, form, features
- UTF-8 encoded
- Some languages have multiple files (e.g., Finnish: `fin.1`, `fin.2`)
- Some have segmentation data (`*.segmentations`)

### Dataset Sizes (examples)
| Language | Forms | Paradigms |
|----------|-------|-----------|
| Italian  | 509k  | 10k       |
| Finnish  | 2.5M  | 58k       |
| Czech    | 50M   | 824k      |
| Polish   | 14M   | 275k      |

### GitHub Structure
Each language is a separate repo: `github.com/unimorph/{iso}`

The main data file is named after the ISO code (e.g., `ita/ita`).

## Contributing to UniMorph

The project welcomes contributions:
- Data corrections via GitHub issues on language repos
- Schema improvements via um-canonicalize repo
- New language data (following annotation guidelines)

All data is CC BY-SA 3.0 licensed.

## References

- [UniMorph Website](https://unimorph.github.io/)
- [UniMorph Schema (Sylak-Glassman 2016)](https://unimorph.github.io/doc/unimorph-schema.pdf)
- [UniMorph 2.0 Paper (LREC 2018)](https://aclanthology.org/L18-1293/)
- [UniMorph 3.0 Paper (LREC 2020)](https://aclanthology.org/2020.lrec-1.483/)
- [SIGMORPHON Shared Tasks](https://sigmorphon.github.io/sharedtasks/)

## License

Apache-2.0 (matching the UniMorph tooling license)
