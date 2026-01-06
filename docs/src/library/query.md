# Query Builder

The query builder provides a fluent interface for flexible searching.

## Basic Usage

```rust
let results = store.query("heb")
    .limit(100)
    .execute()?;
```

## Filter Methods

### By Lemma

```rust
// Exact match
.lemma("כתב")

// LIKE pattern (% = any chars, _ = single char)
.lemma("כת%")      // Starts with כת
.lemma("%ב")       // Ends with ב
.lemma("%בר%")     // Contains בר
.lemma("___")      // Exactly 3 characters
```

### By Form

```rust
// Exact match
.form("כתבתי")

// LIKE pattern
.form("%ים")       // Plural forms ending in ים
.form("ה%")        // Forms starting with ה
```

### By Part of Speech

```rust
.pos("V")          // Verbs
.pos("N")          // Nouns
.pos("ADJ")        // Adjectives
```

### By Features (Pattern Match)

Position-dependent matching with wildcards:

```rust
// Match specific positions
.features_match("V;1;SG;*")      // 1st person singular verbs
.features_match("V;*;*;PST;*")   // Past tense verbs
.features_match("N;*;PL;*")      // Plural nouns
```

### By Features (Contains)

Position-independent matching:

```rust
// Has these features anywhere
.features_contain(&["FUT"])           // Future tense
.features_contain(&["PL", "MASC"])    // Plural masculine
.features_contain(&["V", "1", "SG"])  // 1st person singular verbs
```

## Pagination

```rust
// First page
.limit(20)
.offset(0)

// Second page
.limit(20)
.offset(20)

// All results (careful with large datasets!)
.limit(usize::MAX)
```

## Executing Queries

### Get Results

```rust
let entries: Vec<Entry> = store.query("heb")
    .pos("V")
    .limit(100)
    .execute()?;

for entry in &entries {
    println!("{} {} {}", entry.lemma, entry.form, entry.features);
}
```

### Count Results

```rust
let count = store.query("heb")
    .pos("V")
    .count()?;

println!("Found {} verbs", count);
```

### Check Existence

```rust
let exists = store.query("heb")
    .lemma("כתב")
    .exists()?;

if exists {
    println!("Lemma found");
}
```

### Get First Result

```rust
if let Some(entry) = store.query("heb")
    .lemma("כתב")
    .first()?
{
    println!("First form: {}", entry.form);
}
```

## Chaining Filters

Filters are combined with AND logic:

```rust
let results = store.query("heb")
    .lemma("כת%")                    // AND
    .pos("V")                         // AND
    .features_contain(&["FUT"])       // AND
    .limit(10)
    .execute()?;
```

## Examples

### Find All Verb Infinitives

```rust
let infinitives = store.query("heb")
    .pos("V")
    .features_contain(&["NFIN"])
    .execute()?;
```

### Find Ambiguous Forms

Forms that could be multiple parts of speech:

```rust
let form = "שמר";

let as_verb = store.query("heb")
    .form(form)
    .pos("V")
    .execute()?;

let as_noun = store.query("heb")
    .form(form)
    .pos("N")
    .execute()?;

if !as_verb.is_empty() && !as_noun.is_empty() {
    println!("{} is ambiguous (verb and noun)", form);
}
```

### Paginate Through All Results

```rust
let page_size = 100;
let mut offset = 0;

loop {
    let results = store.query("heb")
        .pos("V")
        .limit(page_size)
        .offset(offset)
        .execute()?;
    
    if results.is_empty() {
        break;
    }
    
    for entry in &results {
        // Process entry
    }
    
    offset += page_size;
}
```

### Export Filtered Subset

```rust
use std::io::Write;

let mut file = std::fs::File::create("verbs.tsv")?;

let verbs = store.query("heb")
    .pos("V")
    .limit(usize::MAX)
    .execute()?;

for entry in &verbs {
    writeln!(file, "{}\t{}\t{}", entry.lemma, entry.form, entry.features)?;
}
```

## Performance Tips

1. **Use limits**: Always set a reasonable limit
2. **Prefer specific filters**: More filters = faster queries
3. **Use `count()` first**: Check result size before fetching all
4. **Index-friendly queries**: Lemma and form queries use indexes

```rust
// Good: Uses index
.lemma("כתב")

// Good: Uses index
.form("כתבתי")

// Slower: Full scan with pattern
.lemma("%תב%")

// Slower: Feature scan
.features_contain(&["FUT"])
```
