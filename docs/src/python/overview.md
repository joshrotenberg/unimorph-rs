# Python Bindings

The `unimorph-rs` Python package provides fast, Rust-powered access to UniMorph morphological data with native Polars DataFrame support.

## Installation

```bash
pip install unimorph-rs
```

For Polars DataFrame support:

```bash
pip install unimorph-rs[polars]
```

**Links:**
- [PyPI Package](https://pypi.org/project/unimorph-rs/)
- [Source Code](https://github.com/joshrotenberg/unimorph-rs/tree/main/crates/unimorph-python)

## Requirements

- Python 3.9+
- [Polars](https://pola.rs/) (optional, for DataFrame methods)

## Quick Start

```python
from unimorph import Store, download

# Download a language dataset (one-time)
download("spa")  # Spanish

# Create a store to query the data
store = Store()

# Get all inflected forms of a lemma
forms = store.inflect("spa", "hablar")
for entry in forms:
    print(f"{entry.form}: {entry.features}")
```

Output:
```
hablar: V;NFIN
hablando: V;V.CVB;PRS
hablado: V;V.PTCP;PST;MASC;SG
hablo: V;IND;PRS;1;SG
hablas: V;IND;PRS;2;SG
habla: V;IND;PRS;3;SG
...
```

## Core API

### download(lang)

Downloads a language dataset from UniMorph. Only needs to be called once per language.

```python
from unimorph import download

download("deu")  # German
download("spa")  # Spanish
download("fra")  # French
```

### Store

The main interface for querying morphological data.

```python
from unimorph import Store

store = Store()
```

#### store.inflect(lang, lemma)

Get all inflected forms for a lemma (dictionary form).

```python
forms = store.inflect("deu", "gehen")  # "to go" in German
for entry in forms:
    print(f"{entry.lemma} -> {entry.form}: {entry.features}")
```

#### store.analyze(lang, form)

Analyze a word form to find possible lemmas and features.

```python
analyses = store.analyze("spa", "hablamos")
for entry in analyses:
    print(f"{entry.form} <- {entry.lemma}: {entry.features}")
```

#### store.search_features(lang, features, limit=None)

Search for entries containing specific morphological features.

```python
# Find all past tense subjunctive forms in Spanish
entries = store.search_features("spa", "SBJV;PST", limit=100)
```

#### store.stats(lang)

Get statistics about a downloaded language dataset.

```python
stats = store.stats("spa")
if stats:
    print(f"Entries: {stats.total_entries}")
    print(f"Unique lemmas: {stats.unique_lemmas}")
    print(f"Unique forms: {stats.unique_forms}")
```

#### store.languages()

List all downloaded languages.

```python
langs = store.languages()
print(langs)  # ['deu', 'ita', 'spa', ...]
```

#### store.has_language(lang)

Check if a language is downloaded.

```python
if store.has_language("fra"):
    print("French data is available")
```

## Polars DataFrame Support

> **Note:** Requires `pip install unimorph-rs[polars]`

All query methods have `_df` variants that return Polars DataFrames for easy data analysis.

```python
from unimorph import Store, download

download("spa")
store = Store()

# Get results as a DataFrame
df = store.inflect_df("spa", "ser")
print(df)
```

Output:
```
shape: (70, 3)
+-------+---------+------------------------+
| lemma | form    | features               |
| ---   | ---     | ---                    |
| str   | str     | str                    |
+-------+---------+------------------------+
| ser   | ser     | V;NFIN                 |
| ser   | siendo  | V;V.CVB;PRS            |
| ser   | sido    | V;V.PTCP;PST;MASC;SG   |
| ser   | soy     | V;IND;PRS;1;SG         |
| ser   | eres    | V;IND;PRS;2;SG         |
| ...   | ...     | ...                    |
+-------+---------+------------------------+
```

### DataFrame Methods

- `store.inflect_df(lang, lemma)` - Inflections as DataFrame
- `store.analyze_df(lang, form)` - Analyses as DataFrame  
- `store.search_features_df(lang, features, limit=None)` - Feature search as DataFrame

### Working with DataFrames

```python
import polars as pl

df = store.inflect_df("spa", "hablar")

# Filter to indicative mood only
indicative = df.filter(pl.col("features").str.contains("IND"))

# Group by tense
by_tense = df.filter(
    pl.col("features").str.contains("IND")
).with_columns(
    pl.when(pl.col("features").str.contains("PRS")).then(pl.lit("present"))
      .when(pl.col("features").str.contains("PST")).then(pl.lit("past"))
      .when(pl.col("features").str.contains("FUT")).then(pl.lit("future"))
      .otherwise(pl.lit("other"))
      .alias("tense")
)

print(by_tense)
```

## Entry Objects

Query results return `Entry` objects with the following attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `lemma` | str | Dictionary form / citation form |
| `form` | str | Inflected surface form |
| `features` | str | UniMorph feature bundle (semicolon-separated) |

```python
entry = store.inflect("spa", "hablar")[0]
print(entry.lemma)     # "hablar"
print(entry.form)      # "hablar"
print(entry.features)  # "V;NFIN"
print(repr(entry))     # Entry(lemma='hablar', form='hablar', features='V;NFIN')
```

## DatasetStats Objects

Statistics returned by `store.stats()`:

| Attribute | Type | Description |
|-----------|------|-------------|
| `language` | str | Language code |
| `total_entries` | int | Total number of entries |
| `unique_lemmas` | int | Number of unique lemmas |
| `unique_forms` | int | Number of unique forms |
| `unique_features` | int | Number of unique feature bundles |

## Example: Building a Conjugation Table

```python
import polars as pl
from unimorph import Store, download

download("spa")
store = Store()

# Get all forms of "hablar" (to speak)
df = store.inflect_df("spa", "hablar")

# Filter to present indicative
present = df.filter(
    pl.col("features").str.contains("IND") & 
    pl.col("features").str.contains("PRS")
)

# Extract person and number
conjugation = present.with_columns([
    pl.when(pl.col("features").str.contains("1")).then(pl.lit("1st"))
      .when(pl.col("features").str.contains("2")).then(pl.lit("2nd"))
      .when(pl.col("features").str.contains("3")).then(pl.lit("3rd"))
      .alias("person"),
    pl.when(pl.col("features").str.contains("SG")).then(pl.lit("singular"))
      .when(pl.col("features").str.contains("PL")).then(pl.lit("plural"))
      .alias("number")
]).select(["person", "number", "form"])

print(conjugation)
```

## See Also

- [UniMorph Feature Schema](../unimorph/schema.md) - Understanding feature codes
- [Available Languages](../unimorph/languages.md) - List of supported languages
- [CLI Reference](../cli/overview.md) - Command-line interface
