# list

List available and cached languages.

**Alias:** `ls`

## Synopsis

```
unimorph list [OPTIONS]
```

## Description

Lists UniMorph languages. By default, shows cached (downloaded) languages with entry counts. Use `--available` to fetch the full list of available languages from GitHub.

## Options

| Option | Description |
|--------|-------------|
| `--cached` | Show only cached (downloaded) languages |
| `--available` | Fetch available languages from GitHub |
| `--refresh` | Refresh the cached list of available languages |
| `--json` | Output as JSON |

## Examples

### List Cached Languages

```bash
unimorph list
```

```
Cached languages:
  fin (2737048 entries)
  heb (33177 entries)

Use 'unimorph list --available' to see all available languages.
```

### List All Available Languages

```bash
unimorph list --available
```

```
Available languages (145 total, 2 cached):

  ady
  afb
  ain
  ...
  heb [cached]
  ...
  zul

Use 'unimorph download <code>' to download a language.
```

### JSON Output

```bash
unimorph list --json
```

```json
["fin", "heb"]
```

```bash
unimorph list --available --json
```

```json
[
  {"code": "ady", "cached": false},
  {"code": "afb", "cached": false},
  ...
  {"code": "heb", "cached": true},
  ...
]
```

### Refresh Available List

```bash
unimorph list --available --refresh
```

Forces a fresh fetch from GitHub (the list is normally cached for 24 hours).

## Pipe-Friendly Output

When piped, outputs just language codes:

```bash
unimorph list | head -3
```

```
fin
heb
```

```bash
# Download all available languages
unimorph list --available | while read lang; do
  unimorph download "$lang"
done
```

## See Also

- [download](./download.md) - Download a language
- [stats](./stats.md) - Show statistics for a language
