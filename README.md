# unimorph-rs

[![Crates.io](https://img.shields.io/crates/v/unimorph)](https://crates.io/crates/unimorph)
[![Documentation](https://img.shields.io/badge/docs-mdBook-blue)](https://joshrotenberg.github.io/unimorph-rs/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

A Rust toolkit for working with [UniMorph](https://unimorph.github.io/) morphological data.

## What is UniMorph?

UniMorph provides morphological paradigm data for 169+ languages in a unified annotation format. Each entry is a triple of lemma, inflected form, and morphological features:

```
lemma       form        features
hablar      hablo       V;IND;PRS;1;SG
hablar      hablado     V;V.PTCP;PST;MASC;SG
ser         soy         V;IND;PRS;1;SG
```

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap joshrotenberg/brew
brew install unimorph
```

### Cargo

```bash
cargo install unimorph
```

### Docker

```bash
docker pull ghcr.io/joshrotenberg/unimorph-rs:latest

# Run with persistent data cache
docker run -v ~/.cache/unimorph:/data ghcr.io/joshrotenberg/unimorph-rs download spa
docker run -v ~/.cache/unimorph:/data ghcr.io/joshrotenberg/unimorph-rs inflect spa hablar
```

### From source

```bash
git clone https://github.com/joshrotenberg/unimorph-rs
cd unimorph-rs
cargo install --path crates/unimorph-cli  # directory still named unimorph-cli
```

## Quick Start

```bash
# Download Spanish dataset
unimorph download spa

# Look up all forms of a verb
unimorph inflect spa hablar

# Analyze a surface form (reverse lookup)
unimorph analyze spa hablo

# Search with filters
unimorph search spa --lemma "habl*" --contains V,IND

# Dataset statistics
unimorph stats spa

# Export to JSON Lines
unimorph export spa -F jsonl -o spanish.jsonl
```

## Library Usage

```rust
use unimorph_core::{Store, Repository, LangCode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Download dataset if needed
    let repo = Repository::new()?;
    let lang: LangCode = "spa".parse()?;
    repo.ensure_dataset(&lang).await?;

    // Query the data
    let store = repo.store()?;
    
    // Get all forms of a lemma
    for entry in store.inflect(&lang, "hablar")? {
        println!("{} -> {} [{}]", entry.lemma, entry.form, entry.features);
    }

    // Reverse lookup: find lemmas for a surface form
    for entry in store.analyze(&lang, "hablo")? {
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
