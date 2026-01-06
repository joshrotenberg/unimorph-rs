# Available Languages

[UniMorph](https://unimorph.github.io/) provides morphological data for 100+ languages. Use `unimorph list --available` to see the current list.

For the complete list of languages with download links, see the [official UniMorph languages page](https://unimorph.github.io/).

## Listing Languages

```bash
# See all available languages
unimorph list --available

# See cached (downloaded) languages
unimorph list --cached

# Refresh the available list
unimorph list --available --refresh
```

## Language Codes

UniMorph uses ISO 639-3 three-letter language codes:

| Code | Language |
|------|----------|
| `ara` | Arabic |
| `deu` | German |
| `ell` | Greek |
| `eng` | English |
| `fas` | Persian |
| `fin` | Finnish |
| `fra` | French |
| `heb` | Hebrew |
| `hin` | Hindi |
| `hun` | Hungarian |
| `ita` | Italian |
| `jpn` | Japanese |
| `kat` | Georgian |
| `kor` | Korean |
| `lat` | Latin |
| `nld` | Dutch |
| `pol` | Polish |
| `por` | Portuguese |
| `ron` | Romanian |
| `rus` | Russian |
| `spa` | Spanish |
| `swe` | Swedish |
| `tur` | Turkish |
| `ukr` | Ukrainian |
| `zho` | Chinese |

And many more...

## Dataset Sizes

Dataset sizes vary significantly:

| Language | Entries | Lemmas |
|----------|---------|--------|
| Finnish (`fin`) | 2.7M+ | 50K+ |
| Spanish (`spa`) | 1.2M+ | 10K+ |
| German (`deu`) | 500K+ | 50K+ |
| Italian (`ita`) | 500K+ | 10K+ |
| Hebrew (`heb`) | 33K+ | 1K+ |

Check specific sizes with:

```bash
unimorph stats <lang>
```

## Language Repositories

Each language has its own GitHub repository under the [UniMorph organization](https://github.com/unimorph):

```
https://github.com/unimorph/<code>
```

For example:
- Hebrew: [github.com/unimorph/heb](https://github.com/unimorph/heb)
- Italian: [github.com/unimorph/ita](https://github.com/unimorph/ita)
- Finnish: [github.com/unimorph/fin](https://github.com/unimorph/fin)

You can also browse all languages on the [UniMorph website](https://unimorph.github.io/).

## Data Quality

Data quality varies by language:

- **High quality**: Languages with extensive Wiktionary coverage
- **Medium quality**: Languages with academic contributions
- **Lower quality**: Newer or less-resourced languages

Check the language's GitHub repository for:
- Data sources
- Known issues
- Contribution guidelines

## Finding Language Codes

If you don't know a language's code:

```bash
# List all available and search
unimorph list --available | grep -i finnish
# Output: fin

# Or use the SIL database
# https://iso639-3.sil.org/code_tables/639/data
```

## Setting Up Aliases

Create shortcuts for frequently used languages:

```toml
# ~/.config/unimorph/config.toml
[languages]
hebrew = "heb"
italian = "ita"
german = "deu"
finnish = "fin"
```

Then use:

```bash
unimorph inflect -l hebrew כתב
# Resolves to: unimorph inflect -l heb כתב
```

## Contributing Languages

To contribute to a language or add a new one:

1. Visit the language repository on [GitHub](https://github.com/unimorph)
2. Check existing issues
3. Submit corrections or additions via pull request

See the [UniMorph contribution guidelines](https://unimorph.github.io/) for more information.
