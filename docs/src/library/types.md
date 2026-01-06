# Types

Core data types in `unimorph-core`.

## LangCode

A validated ISO 639-3 language code (3 lowercase ASCII letters).

```rust
use unimorph_core::LangCode;

// Parse from string
let lang: LangCode = "heb".parse()?;

// Validation happens at parse time
assert!("HEB".parse::<LangCode>().is_err());  // Must be lowercase
assert!("he".parse::<LangCode>().is_err());   // Must be 3 chars
assert!("h3b".parse::<LangCode>().is_err());  // Must be letters

// Convert to string
let s: &str = lang.as_ref();
let s: String = lang.to_string();
```

## Entry

A single morphological entry with lemma, form, and features.

```rust
use unimorph_core::Entry;

// Entries are returned from queries
let entries = store.inflect("heb", "כתב")?;
for entry in entries {
    println!("Lemma: {}", entry.lemma);
    println!("Form: {}", entry.form);
    println!("Features: {}", entry.features);
    println!("Features (raw): {}", entry.features.raw());
    println!("Features (list): {:?}", entry.features.as_slice());
}

// Parse from TSV line
let entry = Entry::parse_line("כתב\tכתבתי\tV;1;SG;PST", 1)?;

// Serialize to JSON
let json = serde_json::to_string(&entry)?;
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `lemma` | `String` | Dictionary form |
| `form` | `String` | Inflected surface form |
| `features` | `FeatureBundle` | Morphological features |

## FeatureBundle

A semicolon-separated bundle of morphological features.

```rust
use unimorph_core::FeatureBundle;

// Parse from string
let features: FeatureBundle = "V;1;SG;PST".parse()?;

// Access individual features
assert_eq!(features.as_slice(), &["V", "1", "SG", "PST"]);
assert_eq!(features.raw(), "V;1;SG;PST");
assert_eq!(features.len(), 4);

// Check if contains a feature (position-independent)
assert!(features.contains("PST"));
assert!(features.contains("V"));
assert!(!features.contains("FUT"));

// Check if contains all features
assert!(features.contains_all(&["V", "PST"]));

// Pattern matching with wildcards
assert!(features.matches("V;*;SG;*"));
assert!(features.matches("V;1;*;PST"));
assert!(!features.matches("N;*;*;*"));

// Display
println!("{}", features);  // "V;1;SG;PST"
```

### Pattern Matching

The `matches` method supports positional pattern matching:

| Pattern | Description |
|---------|-------------|
| `V;1;SG;PST` | Exact match |
| `V;*;SG;*` | Wildcard at positions 1 and 3 |
| `*;*;*;PST` | Only check position 3 |

Note: Pattern must have same number of positions as the bundle.

### Validation

- Feature bundles cannot be empty
- Individual features cannot be empty
- Features are separated by semicolons

```rust
assert!("".parse::<FeatureBundle>().is_err());      // Empty
assert!("V;;SG".parse::<FeatureBundle>().is_err()); // Empty feature
```

## DatasetStats

Statistics about a downloaded language dataset.

```rust
use unimorph_core::DatasetStats;

let stats = store.stats("heb")?;
if let Some(stats) = stats {
    println!("Total entries: {}", stats.total_entries);
    println!("Unique lemmas: {}", stats.unique_lemmas);
    println!("Unique forms: {}", stats.unique_forms);
    println!("Unique features: {}", stats.unique_features);
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_entries` | `usize` | Number of entries |
| `unique_lemmas` | `usize` | Distinct lemmas |
| `unique_forms` | `usize` | Distinct surface forms |
| `unique_features` | `usize` | Distinct feature bundles |

## Serialization

All types implement `Serialize` and `Deserialize` from serde:

```rust
use unimorph_core::Entry;

let entry = store.inflect("heb", "כתב")?.first().unwrap();

// To JSON
let json = serde_json::to_string(&entry)?;

// From JSON
let entry: Entry = serde_json::from_str(&json)?;
```
