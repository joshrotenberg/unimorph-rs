# Repository

The `Repository` manages data downloads, caching, and provides access to the underlying store.

## Creating a Repository

```rust
use unimorph_core::Repository;

// Default location (~/.cache/unimorph)
let repo = Repository::open_default()?;

// Custom location
let repo = Repository::open("/path/to/data")?;

// Custom location with PathBuf
use std::path::PathBuf;
let path = PathBuf::from("/path/to/data");
let repo = Repository::open(&path)?;
```

## Downloading Data

Download a language dataset from UniMorph:

```rust
// Download (async)
repo.download("heb").await?;

// Force re-download
repo.download_with_options("heb", true).await?;
```

### Compressed Files

Some large datasets (Polish, Czech, Ukrainian, etc.) are distributed as compressed `.xz` files due to GitHub file size limits. The repository automatically:

1. Tries compressed versions first (`.xz`, `.gz`)
2. Falls back to uncompressed if not found
3. Decompresses transparently before importing

No special handling is needed - just call `download()` as usual.

## Accessing the Store

Get the underlying store for queries:

```rust
let store = repo.store();

let forms = store.inflect("heb", "כתב")?;
```

## Checking Cached Languages

```rust
// List cached languages
let languages = repo.cached_languages()?;
for lang in &languages {
    println!("Cached: {}", lang);
}

// Check if specific language is cached
if languages.iter().any(|l| l.as_ref() == "heb") {
    println!("Hebrew is cached");
}
```

## Data Directory

The repository manages a data directory containing:

```
~/.cache/unimorph/
├── datasets.db              # SQLite database
└── available_languages.json # Cached API response
```

Get the data directory:

```rust
let data_dir = repo.data_dir();
println!("Data stored in: {}", data_dir.display());
```

## Full Example

```rust
use unimorph_core::Repository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open repository
    let repo = Repository::open_default()?;
    
    // Download Hebrew if not cached
    let cached = repo.cached_languages()?;
    if !cached.iter().any(|l| l.as_ref() == "heb") {
        println!("Downloading Hebrew...");
        repo.download("heb").await?;
    }
    
    // Query the data
    let store = repo.store();
    let forms = store.inflect("heb", "כתב")?;
    
    println!("Found {} forms of כתב:", forms.len());
    for entry in &forms {
        println!("  {} - {}", entry.form, entry.features);
    }
    
    Ok(())
}
```

## Error Handling

```rust
use unimorph_core::{Repository, Error};

async fn download_language(repo: &Repository, lang: &str) -> anyhow::Result<()> {
    match repo.download(lang).await {
        Ok(()) => println!("Downloaded {}", lang),
        Err(Error::Network(e)) => {
            println!("Network error: {}", e);
            println!("Check your connection and try again");
        }
        Err(Error::InvalidLanguage(l)) => {
            println!("Invalid language code: {}", l);
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
```

## Async Runtime

Download operations are async and require a runtime:

```rust
// With tokio
#[tokio::main]
async fn main() {
    let repo = Repository::open_default().unwrap();
    repo.download("heb").await.unwrap();
}

// Or with block_on
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let repo = Repository::open_default().unwrap();
    rt.block_on(repo.download("heb")).unwrap();
}
```
