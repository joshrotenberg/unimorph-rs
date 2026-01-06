# stats

Show dataset statistics.

**Alias:** `st`

## Synopsis

```
unimorph stats [OPTIONS] [LANG]
```

## Description

Displays statistics about a downloaded language dataset, including entry counts, unique lemmas, unique forms, and unique feature combinations.

## Arguments

| Argument | Description |
|----------|-------------|
| `[LANG]` | Language code (ISO 639-3). Optional if default is configured. |

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

## Examples

### Basic Statistics

```bash
unimorph stats heb
```

```
Statistics for heb:
  Total entries:    33177
  Unique lemmas:    1176
  Unique forms:     27286
  Unique features:  55
  Imported at:      2024-01-15 10:30:00 UTC
```

### JSON Output

```bash
unimorph stats heb --json
```

```json
{
  "total_entries": 33177,
  "unique_lemmas": 1176,
  "unique_forms": 27286,
  "unique_features": 55
}
```

### Compare Languages

```bash
for lang in heb ita fin deu; do
  echo "=== $lang ==="
  unimorph stats "$lang"
  echo
done
```

### Scripting

```bash
# Get entry count
unimorph stats heb --json | jq '.total_entries'

# Compare sizes
unimorph list | while read lang; do
  count=$(unimorph stats "$lang" --json | jq '.total_entries')
  echo "$lang: $count"
done | sort -t: -k2 -rn
```

## Understanding the Statistics

| Metric | Description |
|--------|-------------|
| Total entries | Number of (lemma, form, features) triples |
| Unique lemmas | Number of distinct dictionary forms |
| Unique forms | Number of distinct surface forms |
| Unique features | Number of distinct feature bundle combinations |
| Imported at | When the dataset was downloaded |

## See Also

- [info](./info.md) - More detailed language information
- [list](./list.md) - List all cached languages
