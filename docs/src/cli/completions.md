# completions

Generate shell completions for your shell.

## Synopsis

```
unimorph completions <SHELL>
```

## Description

Generates shell completion scripts that enable tab-completion for unimorph commands, options, and arguments.

## Arguments

| Argument | Description |
|----------|-------------|
| `<SHELL>` | Shell to generate completions for: `bash`, `zsh`, `fish`, `elvish`, `powershell` |

## Installation

### Bash

```bash
# Add to ~/.bashrc
source <(unimorph completions bash)

# Or save to a file
unimorph completions bash > ~/.local/share/bash-completion/completions/unimorph
```

### Zsh

```bash
# Add to ~/.zshrc (before compinit)
source <(unimorph completions zsh)

# Or save to fpath
unimorph completions zsh > ~/.zfunc/_unimorph
# Then add to ~/.zshrc: fpath=(~/.zfunc $fpath)
```

### Fish

```bash
unimorph completions fish > ~/.config/fish/completions/unimorph.fish
```

### PowerShell

```powershell
# Add to your PowerShell profile
unimorph completions powershell | Out-String | Invoke-Expression

# Or save to a file and source it
unimorph completions powershell > unimorph.ps1
```

### Elvish

```bash
unimorph completions elvish > ~/.elvish/lib/unimorph.elv
# Then add to ~/.elvish/rc.elv: use unimorph
```

## Examples

After installation, you can use tab completion:

```bash
# Complete commands
unimorph inf<TAB>  # completes to 'inflect'

# Complete options
unimorph inflect --<TAB>  # shows available options

# Complete language codes (if supported by your shell)
unimorph inflect -l <TAB>
```

## Notes

- Restart your shell or source your config file after installation
- Some completions may require a downloaded language list to work

## See Also

- [Installation](../installation.md) - Full installation instructions
