# unimorph-rs

[![Crates.io](https://img.shields.io/crates/v/unimorph-cli)](https://crates.io/crates/unimorph-cli)
[![Documentation](https://img.shields.io/badge/docs-mdBook-blue)](https://joshrotenberg.github.io/unimorph-rs/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

A Rust toolkit for working with [UniMorph](https://unimorph.github.io/) morphological data.

## What is UniMorph?

UniMorph provides morphological paradigm data for 169+ languages in a unified annotation format. Each entry is a triple of lemma, inflected form, and morphological features:

```
lemma       form        features
parlare     parlo       V;IND;PRS;1;SG
parlare     parlato     V.PTCP;PST
essere      sono        V;IND;PRS;1;SG
```

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap joshrotenberg/brew
brew install unimorph
```

### Cargo

```bash
cargo install unimorph-cli
```

### Docker

```bash
docker pull ghcr.io/joshrotenberg/unimorph-rs:latest

# Run with persistent data cache
docker run -v ~/.cache/unimorph:/data ghcr.io/joshrotenberg/unimorph-rs download ita
docker run -v ~/.cache/unimorph:/data ghcr.io/joshrotenberg/unimorph-rs inflect ita parlare
```

### From source

```bash
git clone https://github.com/joshrotenberg/unimorph-rs
cd unimorph-rs
cargo install --path crates/unimorph-cli
```

## Quick Start

```bash
# Download Italian dataset
unimorph download ita

# Look up all forms of a verb
unimorph inflect ita parlare

# Analyze a surface form (reverse lookup)
unimorph analyze ita parlo

# Search with filters
unimorph search ita --lemma "parl*" --contains V,IND

# Dataset statistics
unimorph stats ita

# Export to JSON Lines
unimorph export ita -f jsonl -o italian.jsonl
```

## Library Usage

```rust
use unimorph_core::{Store, Repository, LangCode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Download dataset if needed
    let repo = Repository::new()?;
    let lang: LangCode = "ita".parse()?;
    repo.ensure_dataset(&lang).await?;

    // Query the data
    let store = repo.store()?;
    
    // Get all forms of a lemma
    for entry in store.inflect(&lang, "parlare")? {
        println!("{} -> {} [{}]", entry.lemma, entry.form, entry.features);
    }

    // Reverse lookup: find lemmas for a surface form
    for entry in store.analyze(&lang, "parlo")? {
        println!("{} <- {} [{}]", entry.form, entry.lemma, entry.features);
    }

    Ok(())
}
```

## Documentation

Full documentation is available at **[joshrotenberg.github.io/unimorph-rs](https://joshrotenberg.github.io/unimorph-rs/)**, including:

- [CLI Command Reference](https://joshrotenberg.github.io/unimorph-rs/cli/overview.html)
- [Library API Guide](https://joshrotenberg.github.io/unimorph-rs/library/overview.html)
- [Configuration Options](https://joshrotenberg.github.io/unimorph-rs/configuration.html)
- [UniMorph Schema Reference](https://joshrotenberg.github.io/unimorph-rs/unimorph/schema.html)

## Python Bindings

```bash
pip install unimorph-rs
```

```python
from unimorph import Store, download

download("ita")
store = Store()

for entry in store.inflect("ita", "parlare"):
    print(f"{entry.form}: {entry.features}")
```

See the [Python documentation](https://joshrotenberg.github.io/unimorph-rs/python/overview.html) for more details.

## Project Structure

```
unimorph-rs/
├── crates/
│   ├── unimorph-core/   # Core library: types, SQLite store, repository
│   ├── unimorph-cli/    # Command-line interface
│   └── unimorph-python/ # Python bindings (PyO3)
└── docs/                # mdBook documentation
```

## References

- [UniMorph Website](https://unimorph.github.io/)
- [UniMorph Schema](https://unimorph.github.io/doc/unimorph-schema.pdf)
- [SIGMORPHON Shared Tasks](https://sigmorphon.github.io/sharedtasks/)

## License

Apache-2.0
