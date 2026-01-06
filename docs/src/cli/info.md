# info

Show detailed info about a cached language.

**Alias:** `in`

## Synopsis

```
unimorph info [OPTIONS] [LANG]
```

## Description

Displays detailed information about a downloaded language dataset, including source URL, local and remote commit information, update status, and statistics.

## Arguments

| Argument | Description |
|----------|-------------|
| `[LANG]` | Language code (ISO 639-3). Optional if default is configured. |

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

## Examples

### Basic Info

```bash
unimorph info heb
```

```
Language: heb
Source: https://github.com/unimorph/heb

Local imported:  2024-01-15 10:30:00 UTC
Local commit:    b2bff12
Remote commit:   b2bff12 (2023-01-09)

Status: Up to date

Statistics:
  Total entries:   33177
  Unique lemmas:   1176
  Unique forms:    27286
  Unique features: 55
```

### Update Available

```bash
unimorph info heb
```

```
Language: heb
Source: https://github.com/unimorph/heb

Local imported:  2024-01-15 10:30:00 UTC
Local commit:    b2bff12
Remote commit:   c4d8e23 (2024-02-01)

Status: Update available

Statistics:
  Total entries:   33177
  Unique lemmas:   1176
  Unique forms:    27286
  Unique features: 55
```

### JSON Output

```bash
unimorph info heb --json
```

```json
{
  "language": "heb",
  "source": "https://github.com/unimorph/heb",
  "local_commit": "b2bff12",
  "remote_commit": "c4d8e23",
  "imported_at": "2024-01-15T10:30:00Z",
  "update_available": true,
  "stats": {
    "total_entries": 33177,
    "unique_lemmas": 1176,
    "unique_forms": 27286,
    "unique_features": 55
  }
}
```

## See Also

- [stats](./stats.md) - Quick statistics
- [update](./update.md) - Update datasets
