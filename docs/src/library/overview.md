# Library Overview

The `unimorph-core` crate provides a Rust library for working with UniMorph morphological data. Use it to integrate morphological lookups into your own applications.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unimorph-core = "0.1"
```

## Quick Example

```rust
use unimorph_core::{Repository, LangCode};

fn main() -> anyhow::Result<()> {
    // Create a repository (uses default cache directory)
    let repo = Repository::open_default()?;
    
    // Parse language code
    let lang: LangCode = "heb".parse()?;
    
    // Look up all forms of a lemma
    let forms = repo.store().inflect(&lang, "כתב")?;
    for entry in forms {
        println!("{} -> {} ({})", entry.lemma, entry.form, entry.features);
    }
    
    // Analyze a surface form
    let analyses = repo.store().analyze(&lang, "כתבתי")?;
    for entry in analyses {
        println!("{} <- {} ({})", entry.form, entry.lemma, entry.features);
    }
    
    Ok(())
}
```

## Core Components

### Repository

The `Repository` manages data downloads and caching:

```rust
use unimorph_core::Repository;

// Default location (~/.cache/unimorph)
let repo = Repository::open_default()?;

// Custom location
let repo = Repository::open("/custom/path")?;

// Download a language
repo.download("heb").await?;

// List cached languages
let languages = repo.cached_languages()?;
```

### Store

The `Store` provides the query interface:

```rust
let store = repo.store();

// Inflect: lemma -> forms
let forms = store.inflect("heb", "כתב")?;

// Analyze: form -> lemmas
let analyses = store.analyze("heb", "כתבתי")?;

// Statistics
let stats = store.stats("heb")?;
```

### Query Builder

Flexible searching with the query builder:

```rust
let results = store.query("heb")
    .lemma("כת%")           // LIKE pattern
    .pos("V")                // Part of speech
    .features_contain(&["FUT", "1"])  // Has these features
    .limit(100)
    .execute()?;
```

### Types

Core data types:

```rust
use unimorph_core::{Entry, LangCode, FeatureBundle};

// Language codes (validated)
let lang: LangCode = "heb".parse()?;

// Entries contain lemma, form, features
let entry = Entry {
    lemma: "כתב".to_string(),
    form: "כתבתי".to_string(),
    features: "V;1;SG;PST".parse()?,
};

// Feature bundles support pattern matching
let features: FeatureBundle = "V;1;SG;PST".parse()?;
assert!(features.matches("V;*;SG;*"));
assert!(features.contains("PST"));
```

## Error Handling

The library uses a custom `Error` type:

```rust
use unimorph_core::{Result, Error};

fn example() -> Result<()> {
    let repo = Repository::open_default()?;
    
    match repo.store().inflect("heb", "xyz") {
        Ok(entries) => println!("Found {} entries", entries.len()),
        Err(Error::NotFound(msg)) => println!("Not found: {}", msg),
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `default` | Standard features |
| `parquet` | Parquet export support |

```toml
[dependencies]
unimorph-core = { version = "0.1", features = ["parquet"] }
```

## Next Steps

- [Types](./types.md) - Core data types
- [Store](./store.md) - Query interface
- [Repository](./repository.md) - Data management
- [Query Builder](./query.md) - Advanced searching
