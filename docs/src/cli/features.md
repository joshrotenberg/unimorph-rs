# features

Explore morphological features in a language.

**Alias:** `f`

## Synopsis

```
unimorph features [OPTIONS]
```

## Description

Explore the morphological features used in a language dataset. View unique feature values, their frequencies, search for entries with specific features, or analyze feature positions.

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `--list` | List all unique feature values |
| `--stats` | Show feature value counts (histogram) |
| `--search <FEATURE>` | Search for entries containing a specific feature |
| `--position <N>` | Show values at a specific position (0-indexed) |
| `--limit <N>` | Limit number of results (default: 50) |
| `--json` | Output as JSON |

## Examples

### Feature Structure Overview

```bash
unimorph features -l heb
```

```
Feature structure for heb:

  Position 0: 3 unique values (e.g., V, N, V.MSDR)
  Position 1: 6 unique values (e.g., 2, 3, 1)
  Position 2: 6 unique values (e.g., SG, PL, PRS)
  Position 3: 11 unique values (e.g., FUT, PST, IMP)
  Position 4: 2 unique values (e.g., FEM, MASC)

Use --list for all unique values, --stats for counts, --search <FEATURE> to find entries.
```

### List All Features

```bash
unimorph features -l heb --list
```

```
Unique features in heb:

  1
  2
  3
  DEF
  FEM
  FUT
  IMP
  MASC
  N
  ...

24 unique feature values.
```

### Feature Statistics

```bash
unimorph features -l heb --stats
```

```
Feature statistics for heb:

FEATURE              COUNT
----------------------------------------
V                    28663
SG                   16226
PL                   15158
FEM                  12384
MASC                 12384
2                    12108
FUT                  10400
PST                  9378
3                    7286
1                    4164
... and 14 more
```

### Search by Feature

```bash
unimorph features -l heb --search FUT --limit 5
```

```
Entries with feature 'FUT':

LEMMA                FORM                 FEATURES
------------------------------------------------------------
איבד                 אאבד                 V;1;SG;FUT
איבזר                אאבזר                V;1;SG;FUT
איבטח                אאבטח                V;1;SG;FUT
האביס                אאביס                V;1;SG;FUT
אבל                  אאבל                 V;1;SG;FUT

Showing 5 of 10400 results.
```

### Analyze Feature Position

```bash
unimorph features -l heb --position 0
```

```
Feature values at position 0 in heb:

VALUE                COUNT
----------------------------------------
V                    28663
N                    3338
V.MSDR               1176
```

### JSON Output

```bash
unimorph features -l heb --stats --json
```

```json
{
  "V": 28663,
  "SG": 16226,
  "PL": 15158,
  ...
}
```

### Pipe-Friendly Output

When piped, outputs clean format:

```bash
# Get just feature names
unimorph features -l heb --list | head -5
```

```
1
2
3
DEF
FEM
```

```bash
# Feature counts as TSV
unimorph features -l heb --stats | head -5
```

```
V	28663
SG	16226
PL	15158
FEM	12384
MASC	12384
```

## Use Cases

- **Understanding a language**: See what features are used
- **Finding examples**: Search for entries with specific features
- **Data exploration**: Analyze feature distribution
- **Building queries**: Discover feature names for search filters

## See Also

- [search](./search.md) - Search with feature filters
- [UniMorph Schema](../unimorph/schema.md) - Feature definitions
