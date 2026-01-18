# download

Download a language dataset from UniMorph.

**Alias:** `dl`

## Synopsis

```
unimorph download [OPTIONS] [LANG]
```

## Description

Downloads a UniMorph language dataset from GitHub and imports it into the local SQLite database. Datasets are cached locally, so subsequent queries don't require network access.

If the dataset is already cached, this command does nothing unless `--force` is specified.

## Arguments

| Argument | Description |
|----------|-------------|
| `[LANG]` | Language code (ISO 639-3, e.g., `heb`, `ita`, `deu`). Optional if `UNIMORPH_LANG` is set or configured. |

## Options

| Option | Description |
|--------|-------------|
| `-f, --force` | Force re-download even if cached |
| `--json` | Output as JSON |
| `-q, --quiet` | Suppress progress output |

## Examples

### Basic Download

```bash
unimorph download heb
```

```
Downloading heb...
Downloaded 33177 entries for heb
```

### Force Re-download

```bash
unimorph download heb --force
```

### Quiet Mode

```bash
unimorph download heb --quiet
```

### JSON Output

```bash
unimorph download heb --json
```

```json
{
  "language": "heb",
  "entries": 33177,
  "status": "downloaded"
}
```

### Download Multiple Languages

```bash
for lang in heb ita deu spa; do
  unimorph download "$lang"
done
```

### With Default Language

```bash
export UNIMORPH_LANG=heb
unimorph download  # Downloads Hebrew
```

## Notes

- Language codes are ISO 639-3 (3 lowercase letters)
- Use `unimorph list --available` to see all available languages
- Downloads are atomic: partial downloads won't corrupt your data
- The first download creates the database at `~/.cache/unimorph/datasets.db`
- **Compressed files are handled automatically**: Some large datasets (e.g., Polish, Czech, Ukrainian) are distributed as `.xz` compressed files. The CLI transparently downloads and decompresses these.

## See Also

- [list](./list.md) - List available languages
- [update](./update.md) - Update existing downloads
- [delete](./delete.md) - Remove downloaded data
