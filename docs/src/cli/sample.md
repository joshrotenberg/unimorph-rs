# sample

Randomly sample entries from a language dataset.

**Alias:** `rand`

## Synopsis

```
unimorph sample [OPTIONS] <N>
```

## Description

Samples random entries from a downloaded language dataset. Useful for exploring data, creating test sets, or getting a quick overview of a language's morphology.

## Arguments

| Argument | Description |
|----------|-------------|
| `<N>` | Number of entries to sample |

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `-s, --seed <SEED>` | Seed for reproducible sampling |
| `--by-lemma` | Sample complete paradigms instead of random entries |
| `--json` | Output as JSON |
| `--tsv` | Output as TSV (tab-separated, no headers) |

## Examples

### Random Entries

```bash
unimorph sample -l spa 5
```

```
LEMMA        FORM         FEATURES
------------------------------------------------------------
tapiar      tapiemos     V;SBJV;PRS;1;PL
apilar      apilando     V;V.CVB;PRS
hablar      hablaste     V;IND;PST;PFV;2;SG;INFM
comer       comieron     V;IND;PST;PFV;3;PL
vivir       viviremos    V;IND;FUT;1;PL

5 sampled entry(ies).
```

### Sample Complete Paradigms

Use `--by-lemma` to get all forms of randomly selected lemmas:

```bash
unimorph sample -l spa 2 --by-lemma
```

This returns complete paradigms for 2 random lemmas, showing all their inflected forms.

### Reproducible Sampling

Use `--seed` for reproducible results:

```bash
unimorph sample -l spa 5 --seed 42
```

Running with the same seed always returns the same entries.

### JSON Output

```bash
unimorph sample -l spa 3 --json
```

```json
[
  {
    "lemma": "hablar",
    "form": "hablamos",
    "features": {
      "raw": "V;IND;PRS;1;PL",
      "features": ["V", "IND", "PRS", "1", "PL"]
    }
  },
  ...
]
```

### TSV for Scripting

```bash
unimorph sample -l spa 10 --tsv > sample.tsv
```

### Scripting Examples

```bash
# Create a test set
unimorph sample -l spa 100 --seed 123 --tsv > test_set.tsv

# Sample paradigms for flashcard generation
unimorph sample -l spa 10 --by-lemma --json > flashcards.json

# Get random verbs only
unimorph sample -l spa 50 --tsv | grep "^V;" | head -10
```

## Notes

- Without `--seed`, results are different each run
- `--by-lemma` returns more entries than N (all forms of N lemmas)
- Large N values may take longer for big datasets

## See Also

- [search](./search.md) - Find specific entries
- [inflect](./inflect.md) - Look up forms for a known lemma
