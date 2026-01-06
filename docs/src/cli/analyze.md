# analyze

Analyze a surface form (reverse lookup).

**Alias:** `a`

## Synopsis

```
unimorph analyze [OPTIONS] <FORM>
```

## Description

Given a surface form (inflected word), returns all possible analyses: the lemma it comes from and its morphological features. This is the reverse of `inflect`.

A form may have multiple analyses if it's ambiguous (e.g., same spelling for different lemmas or different grammatical analyses).

## Arguments

| Argument | Description |
|----------|-------------|
| `<FORM>` | The surface form to analyze |

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `--json` | Output as JSON |
| `--tsv` | Output as TSV (tab-separated, no headers) |

## Examples

### Basic Analysis

```bash
unimorph analyze -l heb כתבתי
```

```
FORM         LEMMA        FEATURES
------------------------------------------------------------
כתבתי       כתב         V;1;SG;PST

1 analysis(es) found.
```

### Ambiguous Forms

Some forms have multiple possible analyses:

```bash
unimorph analyze -l heb כתבו
```

```
FORM         LEMMA        FEATURES
------------------------------------------------------------
כתבו        כתב         V;3;PL;PST
כתבו        כתב         V;2;PL;IMP;MASC

2 analysis(es) found.
```

### JSON Output

```bash
unimorph analyze -l heb כתבתי --json
```

```json
[
  {
    "lemma": "כתב",
    "form": "כתבתי",
    "features": {
      "raw": "V;1;SG;PST",
      "features": ["V", "1", "SG", "PST"]
    }
  }
]
```

### TSV for Piping

```bash
unimorph analyze -l heb כתבתי --tsv
```

```
כתבתי	כתב	V;1;SG;PST
```

### Form Not Found

```bash
unimorph analyze -l heb xyz
```

```
No analyses found for 'xyz'.

The form may not exist in the dataset, or it could be:
  - A proper noun or foreign word
  - A misspelling
  - A rare or archaic form
```

### Scripting Examples

```bash
# Analyze words from a file
cat words.txt | while read word; do
  echo "=== $word ==="
  unimorph analyze -l heb "$word"
done

# Get just the lemma
unimorph analyze -l heb כתבתי --tsv | cut -f2

# Check if a word exists
if unimorph analyze -l heb כתבתי --tsv | grep -q .; then
  echo "Found"
fi
```

## Notes

- Analysis is case-sensitive for most languages
- Forms must match exactly (no fuzzy matching)
- Use [search](./search.md) with `--form` for pattern matching

## See Also

- [inflect](./inflect.md) - Forward lookup (lemma to forms)
- [search](./search.md) - Flexible searching
