# Installation

## Command-Line Tool

### Homebrew (macOS/Linux)

```bash
brew tap joshrotenberg/brew
brew install unimorph
```

### Cargo (from crates.io)

If you have Rust installed:

```bash
cargo install unimorph
```

### From Source

```bash
git clone https://github.com/joshrotenberg/unimorph-rs
cd unimorph-rs
cargo install --path crates/unimorph-cli  # directory still named unimorph-cli
```

## Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
unimorph-core = "0.1"
```

Or with cargo:

```bash
cargo add unimorph-core
```

## Shell Completions

Generate completions for your shell:

```bash
# Bash
unimorph completions bash > ~/.local/share/bash-completion/completions/unimorph

# Zsh
unimorph completions zsh > ~/.zfunc/_unimorph

# Fish
unimorph completions fish > ~/.config/fish/completions/unimorph.fish

# PowerShell
unimorph completions powershell > _unimorph.ps1
```

For Zsh, ensure `~/.zfunc` is in your `fpath`:

```bash
# Add to ~/.zshrc before compinit
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

## Verifying Installation

```bash
unimorph --version
unimorph --help
```

## Data Storage

By default, unimorph stores data in:

- **Linux/macOS**: `~/.cache/unimorph/`
- **Custom**: Set `UNIMORPH_DATA` environment variable or use `--data-dir`

Configuration is stored in:

- **All platforms**: `~/.config/unimorph/config.toml`
