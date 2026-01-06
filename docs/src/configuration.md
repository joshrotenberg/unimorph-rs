# Configuration

unimorph can be configured through environment variables, a config file, or command-line flags. Settings are applied in this priority order (highest to lowest):

1. Command-line flags
2. Environment variables
3. Config file
4. Built-in defaults

## Config File

The config file is located at `~/.config/unimorph/config.toml` on all platforms.

### Creating a Config File

```bash
# Create a config file with example content
unimorph config init

# View current configuration
unimorph config show

# Show config file path
unimorph config path
```

### Config File Format

```toml
# Default language for commands (ISO 639-3 code)
default_lang = "heb"

# Custom data directory (default: ~/.cache/unimorph)
# data_dir = "/path/to/custom/data"

# Default output format: "table", "json", or "tsv"
# output_format = "table"

# Disable colored output
# no_color = true

# Language aliases for convenience
[languages]
hebrew = "heb"
spanish = "spa"
german = "deu"
spanish = "spa"
finnish = "fin"
```

### Language Aliases

Define shortcuts for language codes:

```toml
[languages]
he = "heb"
it = "spa"
de = "deu"
```

Then use:

```bash
unimorph inflect -l he כתב
# Resolves to: unimorph inflect -l heb כתב
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `UNIMORPH_LANG` | Default language code | `export UNIMORPH_LANG=heb` |
| `UNIMORPH_DATA` | Custom data directory | `export UNIMORPH_DATA=/data/unimorph` |
| `NO_COLOR` | Disable colored output | `export NO_COLOR=1` |

## Command-Line Flags

Global flags available on all commands:

| Flag | Description |
|------|-------------|
| `-d, --data-dir <PATH>` | Custom data directory |
| `-v, --verbose` | Enable debug output (`-vv` for trace) |
| `-q, --quiet` | Suppress non-essential output |

## Data Storage

### Default Locations

- **Dataset database**: `~/.cache/unimorph/datasets.db`
- **API cache**: `~/.cache/unimorph/available_languages.json`
- **Config file**: `~/.config/unimorph/config.toml`

### Custom Data Directory

Override the data directory:

```bash
# Via environment variable
export UNIMORPH_DATA=/custom/path
unimorph download heb

# Via command-line flag
unimorph --data-dir /custom/path download heb

# Via config file
# data_dir = "/custom/path"
```

### Resetting Data

```bash
# Clear API response cache
unimorph repair --clear-cache

# Clear all downloaded datasets (requires re-download)
unimorph repair --clear-data
```

## Output Modes

### Table (Default)

Human-readable formatted output with colors when connected to a terminal:

```bash
unimorph inflect -l heb כתב
```

### JSON

Machine-readable JSON output:

```bash
unimorph inflect -l heb כתב --json
```

### TSV

Tab-separated values without headers, ideal for piping:

```bash
unimorph inflect -l heb כתב --tsv
```

### Pipe Detection

When stdout is not a terminal (e.g., piped to another command), unimorph automatically outputs in a pipe-friendly format:

```bash
# Automatically outputs just language codes, one per line
unimorph list | xargs -I{} echo "Language: {}"
```
