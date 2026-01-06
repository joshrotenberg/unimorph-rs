# config

Manage configuration.

**Alias:** `cfg`

## Synopsis

```
unimorph config <COMMAND>
```

## Subcommands

| Command | Description |
|---------|-------------|
| `show` | Show current configuration |
| `init` | Initialize a new config file |
| `path` | Show the config file path |

## config show

Display the current configuration, including both config file settings and defaults.

```bash
unimorph config show
```

```
Configuration

  Path: /home/user/.config/unimorph/config.toml
  Status: loaded

Current Settings

  default_lang: heb
  data_dir: (default)
  output_format: (default: table)
  no_color: (not set)
```

### JSON Output

```bash
unimorph config show --json
```

```json
{
  "path": "/home/user/.config/unimorph/config.toml",
  "exists": true,
  "default_lang": "heb",
  "data_dir": null,
  "output_format": null,
  "no_color": null
}
```

## config init

Create a new config file with example content.

```bash
unimorph config init
```

```
Created config file at /home/user/.config/unimorph/config.toml
```

### Force Overwrite

```bash
unimorph config init --force
```

Overwrites existing config file.

### JSON Output

```bash
unimorph config init --json
```

```json
{
  "path": "/home/user/.config/unimorph/config.toml",
  "created": true
}
```

## config path

Show the config file path.

```bash
unimorph config path
```

```
/home/user/.config/unimorph/config.toml
```

### JSON Output

```bash
unimorph config path --json
```

```json
{
  "path": "/home/user/.config/unimorph/config.toml"
}
```

## Config File Format

The config file uses TOML format:

```toml
# Default language for commands
default_lang = "heb"

# Custom data directory
# data_dir = "/custom/path"

# Default output format: "table", "json", or "tsv"
# output_format = "table"

# Disable colored output
# no_color = true

# Language aliases
[languages]
hebrew = "heb"
italian = "ita"
```

## See Also

- [Configuration Guide](../configuration.md) - Full configuration documentation
