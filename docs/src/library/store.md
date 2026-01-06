# Store

The `Store` provides the query interface for morphological data.

## Opening a Store

Usually accessed through `Repository`:

```rust
use unimorph_core::Repository;

let repo = Repository::open_default()?;
let store = repo.store();
```

Or open directly:

```rust
use unimorph_core::Store;

// Open existing database
let store = Store::open("path/to/datasets.db")?;

// In-memory store (for testing)
let store = Store::in_memory()?;
```

## Basic Queries

### Inflect (Lemma to Forms)

Look up all inflected forms of a lemma:

```rust
let forms = store.inflect("heb", "כתב")?;

for entry in &forms {
    println!("{} -> {} ({})", entry.lemma, entry.form, entry.features);
}

println!("Found {} forms", forms.len());
```

### Analyze (Form to Lemmas)

Find all possible lemmas for a surface form:

```rust
let analyses = store.analyze("heb", "כתבו")?;

for entry in &analyses {
    println!("{} <- {} ({})", entry.form, entry.lemma, entry.features);
}

// Handle ambiguous forms
if analyses.len() > 1 {
    println!("Ambiguous: {} possible analyses", analyses.len());
}
```

### Statistics

Get dataset statistics:

```rust
if let Some(stats) = store.stats("heb")? {
    println!("Entries: {}", stats.total_entries);
    println!("Lemmas: {}", stats.unique_lemmas);
    println!("Forms: {}", stats.unique_forms);
}
```

### Check Language

Check if a language is loaded:

```rust
if store.has_language("heb")? {
    println!("Hebrew is available");
}

// List all languages
let languages = store.languages()?;
for lang in languages {
    println!("- {}", lang);
}
```

## Query Builder

For flexible searching, use the query builder:

```rust
let results = store.query("heb")
    .lemma("כת%")           // LIKE pattern (% = any chars)
    .form("%ים")            // Forms ending in ים
    .pos("V")               // Part of speech
    .features_match("V;*;SG;*")  // Pattern match
    .features_contain(&["FUT"])  // Contains feature
    .limit(100)
    .offset(0)
    .execute()?;
```

See [Query Builder](./query.md) for full documentation.

## Data Management

### Import Data

Import entries from TSV format:

```rust
use unimorph_core::{Entry, LangCode};

let lang: LangCode = "test".parse()?;
let entries = vec![
    Entry::parse_line("test\tform1\tN;SG", 1)?,
    Entry::parse_line("test\tform2\tN;PL", 2)?,
];

store.import(&lang, &entries, None, None)?;
```

### Delete Language

Remove a language from the store:

```rust
let removed = store.delete_language("heb")?;
println!("Removed {} entries", removed);
```

## Export

Export to various formats:

```rust
// Export to TSV file
let count = store.export_tsv("heb", "hebrew.tsv")?;

// Export to JSONL file
let count = store.export_jsonl("heb", "hebrew.jsonl")?;

// Export to writer (e.g., stdout)
use std::io::stdout;
let count = store.export_tsv_to_writer("heb", stdout().lock())?;

// Parquet (with feature flag)
#[cfg(feature = "parquet")]
let count = store.export_parquet("heb", "hebrew.parquet")?;
```

## Thread Safety

`Store` is `Send` but not `Sync`. For concurrent access, use a mutex or create separate store instances:

```rust
use std::sync::Mutex;

let store = Mutex::new(Store::open("datasets.db")?);

// In threads:
let store = store.lock().unwrap();
let results = store.inflect("heb", "כתב")?;
```

## Error Handling

```rust
use unimorph_core::{Store, Error};

match store.inflect("xyz", "test") {
    Ok(entries) => println!("Found {} entries", entries.len()),
    Err(Error::LanguageNotFound(lang)) => {
        println!("Language {} not downloaded", lang);
    }
    Err(e) => return Err(e.into()),
}
```
