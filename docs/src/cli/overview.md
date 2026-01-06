# CLI Overview

The `unimorph` command-line tool provides access to UniMorph morphological data through a set of intuitive subcommands.

## Command Structure

```
unimorph [OPTIONS] <COMMAND> [ARGS]
```

### Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable debug output (`-vv` for trace) |
| `-q, --quiet` | Suppress non-essential output |
| `-d, --data-dir <PATH>` | Custom data directory |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Commands at a Glance

| Command | Alias | Description |
|---------|-------|-------------|
| [download](./download.md) | `dl` | Download a language dataset |
| [list](./list.md) | `ls` | List available/cached languages |
| [inflect](./inflect.md) | `i` | Look up all forms of a lemma |
| [analyze](./analyze.md) | `a` | Analyze a surface form (reverse lookup) |
| [search](./search.md) | `s` | Search entries with flexible filtering |
| [stats](./stats.md) | `st` | Show dataset statistics |
| [info](./info.md) | `in` | Show detailed info about a language |
| [export](./export.md) | `x` | Export dataset to file |
| [update](./update.md) | `up` | Update cached datasets |
| [features](./features.md) | `f` | Explore morphological features |
| [delete](./delete.md) | `rm` | Delete a cached dataset |
| [repair](./repair.md) | | Repair or reset data store |
| [config](./config.md) | `cfg` | Manage configuration |
| completions | | Generate shell completions |

## Common Workflows

### First-Time Setup

```bash
# See what languages are available
unimorph list --available

# Download a language
unimorph download heb

# Set as default (optional)
export UNIMORPH_LANG=heb
```

### Looking Up Words

```bash
# All forms of a lemma
unimorph inflect -l heb כתב

# What lemma does this form come from?
unimorph analyze -l heb כתבתי
```

### Searching

```bash
# By features
unimorph search -l heb --contains PL,MASC

# By part of speech
unimorph search -l heb --pos V --limit 20

# By lemma pattern
unimorph search -l heb --lemma "כת%"
```

### Data Management

```bash
# Check for updates
unimorph update --all --check

# Update a specific language
unimorph update heb

# Export for external use
unimorph export -l heb -o hebrew.tsv
```

## Output Formats

Most commands support multiple output formats:

| Flag | Format | Use Case |
|------|--------|----------|
| (default) | Table | Human reading in terminal |
| `--json` | JSON | Machine parsing, APIs |
| `--tsv` | TSV | Piping to other tools |

### Examples

```bash
# Pretty table output
unimorph inflect -l heb כתב

# JSON for parsing
unimorph inflect -l heb כתב --json | jq '.[0]'

# TSV for piping
unimorph inflect -l heb כתב --tsv | cut -f2 | sort -u
```

## Piping and Scripting

When output is piped (not a terminal), unimorph automatically uses pipe-friendly formats:

```bash
# Get all cached language codes
unimorph list | while read lang; do
  echo "Processing $lang..."
  unimorph stats "$lang"
done

# Export to stdout and filter
unimorph export -l heb -o - | grep "FUT" > future_forms.tsv

# Count forms per lemma
unimorph search -l heb --pos V --tsv --limit 1000 | cut -f1 | sort | uniq -c | sort -rn | head
```

## Error Handling

Commands provide helpful error messages:

```bash
$ unimorph inflect כתב
Error: No language specified.

Provide a language code as an argument, or set a default:

  export UNIMORPH_LANG=heb

Or in ~/.config/unimorph/config.toml:

  default_lang = "heb"

Run 'unimorph list --available' to see available languages.
```

## Getting Help

```bash
# General help
unimorph --help

# Command-specific help
unimorph inflect --help
unimorph search --help
```
