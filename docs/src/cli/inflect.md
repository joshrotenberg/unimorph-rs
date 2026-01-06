# inflect

Look up all inflected forms of a lemma.

**Alias:** `i`

## Synopsis

```
unimorph inflect [OPTIONS] <LEMMA>
```

## Description

Given a lemma (dictionary form), returns all its inflected forms with their morphological features. This is the primary way to see a word's full paradigm.

## Arguments

| Argument | Description |
|----------|-------------|
| `<LEMMA>` | The lemma (dictionary form) to look up |

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `-f, --features <PATTERN>` | Filter by feature pattern (e.g., `V;IND;*;SG`) |
| `--json` | Output as JSON |
| `--tsv` | Output as TSV (tab-separated, no headers) |

## Examples

### Basic Lookup

```bash
unimorph inflect -l heb כתב
```

```
LEMMA        FORM         FEATURES
------------------------------------------------------------
כתב         אכתוב       V;1;SG;FUT
כתב         יכתבו       V;3;PL;FUT;MASC
כתב         יכתוב       V;3;SG;FUT;MASC
כתב         כותב        V;SG;PRS;MASC
כתב         כתב         V;3;SG;PST;MASC
...

29 form(s) found.
```

### Filter by Features

Use wildcards (`*`) to match any value at a position:

```bash
# Only singular forms
unimorph inflect -l heb כתב -f "V;*;SG;*"

# Only past tense
unimorph inflect -l heb כתב -f "V;*;*;PST;*"
```

### JSON Output

```bash
unimorph inflect -l heb כתב --json
```

```json
[
  {
    "lemma": "כתב",
    "form": "אכתוב",
    "features": {
      "raw": "V;1;SG;FUT",
      "features": ["V", "1", "SG", "FUT"]
    }
  },
  ...
]
```

### TSV for Piping

```bash
unimorph inflect -l heb כתב --tsv
```

```
כתב	אכתוב	V;1;SG;FUT
כתב	יכתבו	V;3;PL;FUT;MASC
כתב	יכתוב	V;3;SG;FUT;MASC
...
```

### Scripting Examples

```bash
# Get unique forms only
unimorph inflect -l heb כתב --tsv | cut -f2 | sort -u

# Count forms by tense
unimorph inflect -l heb כתב --tsv | cut -f3 | grep -o 'PST\|PRS\|FUT' | sort | uniq -c

# Find forms matching a pattern
unimorph inflect -l ita hablar --tsv | grep "1;SG"
```

## Notes

- The lemma must match exactly (case-sensitive for most languages)
- Use [search](./search.md) with `--lemma` for partial/wildcard matching
- Returns empty results if the lemma doesn't exist in the dataset

## See Also

- [analyze](./analyze.md) - Reverse lookup (form to lemma)
- [search](./search.md) - Flexible searching
