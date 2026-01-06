# update

Update cached language datasets.

**Alias:** `up`

## Synopsis

```
unimorph update [OPTIONS] [LANG]
```

## Description

Checks for and downloads updates to cached language datasets. Can update a single language or all cached languages at once.

## Arguments

| Argument | Description |
|----------|-------------|
| `[LANG]` | Language code to update. Omit with `--all` to update all. |

## Options

| Option | Description |
|--------|-------------|
| `--all` | Update all cached languages |
| `--check` | Check for updates without downloading |
| `--json` | Output as JSON |

## Examples

### Check for Updates

```bash
unimorph update heb --check
```

```
Checking for updates...

  heb - update available
```

Or if up to date:

```
Checking for updates...

  heb - up to date
```

### Update a Single Language

```bash
unimorph update heb
```

```
Updating heb...
Updated heb: 33177 -> 33250 entries
```

### Check All Languages

```bash
unimorph update --all --check
```

```
Checking for updates...

  fin - up to date
  heb - update available
  ita - up to date

1 update(s) available.
```

### Update All Languages

```bash
unimorph update --all
```

```
Updating all cached languages...

  fin - up to date
  heb - updated (33177 -> 33250 entries)
  ita - up to date

1 language(s) updated.
```

### JSON Output

```bash
unimorph update --all --check --json
```

```json
{
  "languages": [
    {"code": "fin", "update_available": false},
    {"code": "heb", "update_available": true},
    {"code": "spa", "update_available": false}
  ],
  "updates_available": 1
}
```

### Scripting

```bash
# Check and update only if needed
if unimorph update heb --check --json | jq -e '.update_available' > /dev/null; then
  unimorph update heb
fi
```

## See Also

- [info](./info.md) - View update status
- [download](./download.md) - Initial download
