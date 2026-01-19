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

## Verbose Output

Use `-v` for detailed import reporting:

```bash
unimorph download spa --force -v
```

This shows:

```
parsed downloaded data lang=spa filename=["spa"] compression=none from_lfs=false valid_entries=1196224 blank_lines=0 malformed=21
malformed entry lang=spa line=80710 reason=empty form
malformed entry lang=spa line=134234 reason=empty form
...
additional malformed entries not shown lang=spa additional=11
```

### Understanding the Output

| Field | Description |
|-------|-------------|
| `filename` | Source file(s) downloaded |
| `compression` | Format: `none`, `xz`, `gzip`, or `zip` |
| `from_lfs` | Whether fetched via Git LFS (large files) |
| `valid_entries` | Successfully parsed entries |
| `blank_lines` | Empty lines skipped (not an error) |
| `malformed` | Entries that failed to parse |

### Malformed Entry Details

When entries fail to parse, the first 10 are logged with:
- **Line number**: Where in the source file
- **Reason**: Why it failed (e.g., "empty form", "expected at least 3 columns")

Common reasons for malformed entries:
- `empty form` - The inflected form field is blank
- `empty lemma` - The dictionary form field is blank
- `expected at least 3 columns` - Line doesn't have lemma, form, and features

These indicate upstream data quality issues in the UniMorph repository.

## Notes

- Language codes are ISO 639-3 (3 lowercase letters)
- Use `unimorph list --available` to see all available languages
- Downloads are atomic: partial downloads won't corrupt your data
- The first download creates the database at `~/.cache/unimorph/datasets.db`
- **Compressed files**: Large datasets (Polish, Czech, Ukrainian, Slovak) use `.xz` compression - handled automatically
- **Git LFS**: Very large files (like Czech's full MorfFlex dataset) use Git LFS - also handled automatically

## See Also

- [list](./list.md) - List available languages
- [update](./update.md) - Update existing downloads
- [delete](./delete.md) - Remove downloaded data
