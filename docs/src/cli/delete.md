# delete

Delete a cached language dataset.

**Alias:** `rm`

## Synopsis

```
unimorph delete [OPTIONS] [LANG]
```

## Description

Removes a downloaded language dataset from the local cache. The data can be re-downloaded later with `unimorph download`.

## Arguments

| Argument | Description |
|----------|-------------|
| `[LANG]` | Language code (ISO 639-3). Optional if default is configured. |

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

## Examples

### Delete a Language

```bash
unimorph delete heb
```

```
Deleted heb (33177 entries removed)
```

### JSON Output

```bash
unimorph delete heb --json
```

```json
{
  "language": "heb",
  "entries_removed": 33177,
  "status": "deleted"
}
```

### Delete Multiple Languages

```bash
for lang in heb ita deu; do
  unimorph delete "$lang"
done
```

## Notes

- This only removes the data from the local cache
- Statistics and metadata are also removed
- Re-download anytime with `unimorph download`
- Use `unimorph repair --clear-data` to delete all languages at once

## See Also

- [download](./download.md) - Re-download data
- [repair](./repair.md) - Clear all data
- [list](./list.md) - See cached languages
