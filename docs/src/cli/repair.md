# repair

Repair or reset the local data store.

## Synopsis

```
unimorph repair [OPTIONS]
```

## Description

Utility command for troubleshooting and resetting the local data store. Can clear the API response cache or all downloaded datasets.

## Options

| Option | Description |
|--------|-------------|
| `--clear-cache` | Clear cached API responses |
| `--clear-data` | Clear all downloaded datasets (requires re-download) |
| `--json` | Output as JSON |

## Examples

### Clear API Cache

Clears the cached list of available languages (normally cached for 24 hours):

```bash
unimorph repair --clear-cache
```

```
Cleared API cache
```

### Clear All Data

Removes all downloaded language datasets:

```bash
unimorph repair --clear-data
```

```
Cleared all data (5 languages removed)
```

### Clear Both

```bash
unimorph repair --clear-cache --clear-data
```

### JSON Output

```bash
unimorph repair --clear-data --json
```

```json
{
  "cache_cleared": false,
  "data_cleared": true,
  "languages_removed": 5
}
```

## Use Cases

- **Corrupted data**: If queries return unexpected results
- **Stale cache**: If available language list seems outdated
- **Disk space**: Remove all data to free space
- **Fresh start**: Reset everything to initial state

## Notes

- `--clear-cache` only removes API response cache, not datasets
- `--clear-data` removes all downloaded languages
- Data can be re-downloaded with `unimorph download`

## See Also

- [delete](./delete.md) - Delete individual languages
- [download](./download.md) - Re-download data
