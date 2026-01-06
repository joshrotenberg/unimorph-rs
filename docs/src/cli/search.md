# search

Search entries with flexible filtering.

**Alias:** `s`

## Synopsis

```
unimorph search [OPTIONS]
```

## Description

Search the dataset with flexible filtering by lemma, form, features, part of speech, and more. Supports wildcards and multiple filter combinations.

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `--lemma <PATTERN>` | Filter by lemma (supports SQL LIKE wildcards: `%` and `_`) |
| `--form <PATTERN>` | Filter by form (supports SQL LIKE wildcards) |
| `-f, --features <PATTERN>` | Filter by feature pattern (e.g., `V;IND;*;1;*`) |
| `-c, --contains <FEATURES>` | Filter by features contained (comma-separated, position-independent) |
| `--pos <POS>` | Filter by part of speech (e.g., `V`, `N`, `ADJ`) |
| `--limit <N>` | Limit number of results (default: 100) |
| `--offset <N>` | Skip first N results |
| `--count` | Just show count of matching entries |
| `--json` | Output as JSON |
| `--tsv` | Output as TSV |

## Examples

### Search by Lemma Pattern

```bash
# Lemmas starting with "כת"
unimorph search -l heb --lemma "כת%"

# Lemmas containing "בר"
unimorph search -l heb --lemma "%בר%"

# Exact 4-letter lemmas
unimorph search -l heb --lemma "____"
```

### Search by Form Pattern

```bash
# Forms ending with "ים"
unimorph search -l heb --form "%ים"
```

### Filter by Features (Position-Dependent)

Use semicolon-separated patterns with `*` as wildcard:

```bash
# First person singular verbs
unimorph search -l heb -f "V;1;SG;*"

# Past tense forms
unimorph search -l heb -f "V;*;*;PST;*"
```

### Filter by Features (Position-Independent)

Use `--contains` for features that can be at any position:

```bash
# Plural masculine forms (regardless of position)
unimorph search -l heb --contains PL,MASC

# Future tense first person
unimorph search -l heb --contains FUT,1
```

### Filter by Part of Speech

```bash
# Only verbs
unimorph search -l heb --pos V

# Only nouns
unimorph search -l heb --pos N
```

### Combine Filters

```bash
# Verbs with plural masculine future forms
unimorph search -l heb --pos V --contains PL,MASC,FUT

# Lemmas starting with "א" that are verbs
unimorph search -l heb --lemma "א%" --pos V
```

### Pagination

```bash
# First 20 results
unimorph search -l heb --pos V --limit 20

# Results 21-40
unimorph search -l heb --pos V --limit 20 --offset 20
```

### Count Only

```bash
unimorph search -l heb --pos V --count
```

```
15234 entries match.
```

### Output Formats

```bash
# JSON
unimorph search -l heb --pos V --limit 5 --json

# TSV for piping
unimorph search -l heb --pos V --limit 5 --tsv
```

### Scripting Examples

```bash
# Get unique lemmas for a part of speech
unimorph search -l heb --pos V --limit 10000 --tsv | cut -f1 | sort -u

# Count entries per lemma
unimorph search -l heb --pos V --limit 10000 --tsv | cut -f1 | sort | uniq -c | sort -rn | head

# Export filtered subset
unimorph search -l heb --contains FUT --tsv > future_forms.tsv
```

## Wildcards Reference

### SQL LIKE Wildcards (for `--lemma` and `--form`)

| Pattern | Matches |
|---------|---------|
| `%` | Any sequence of characters |
| `_` | Any single character |
| `abc%` | Starts with "abc" |
| `%abc` | Ends with "abc" |
| `%abc%` | Contains "abc" |
| `a_c` | "a" + any char + "c" |

### Feature Pattern Wildcards (for `-f`)

| Pattern | Matches |
|---------|---------|
| `*` | Any value at that position |
| `V;*;SG;*` | Verb, any person, singular, any tense |

## See Also

- [inflect](./inflect.md) - Exact lemma lookup
- [analyze](./analyze.md) - Exact form lookup
- [features](./features.md) - Explore available features
