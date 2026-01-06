# unimorph-rs

Python bindings for the UniMorph morphological data toolkit.

## Installation

```bash
pip install unimorph-rs
```

For optional Polars DataFrame support:

```bash
pip install unimorph-rs[polars]
```

## Usage

```python
from unimorph import Store, download

# Download Italian data
download("ita")

# Create a store
store = Store()

# Get all inflected forms of a lemma
forms = store.inflect("ita", "parlare")
for entry in forms:
    print(f"{entry.form}: {entry.features}")

# Analyze a word form
analyses = store.analyze("ita", "parlo")

# Get results as Polars DataFrames
df = store.inflect_df("ita", "parlare")
print(df)
```

## Features

- Fast Rust-powered morphological lookups
- Polars DataFrame integration
- Support for 100+ languages from UniMorph
