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

# Download Spanish data
download("spa")

# Create a store
store = Store()

# Get all inflected forms of a lemma
forms = store.inflect("spa", "hablar")
for entry in forms:
    print(f"{entry.form}: {entry.features}")

# Analyze a word form
analyses = store.analyze("spa", "hablo")

# Get results as Polars DataFrames (requires unimorph-rs[polars])
df = store.inflect_df("spa", "hablar")
print(df)
```

## Features

- Fast Rust-powered morphological lookups
- Polars DataFrame integration
- Support for 100+ languages from UniMorph
