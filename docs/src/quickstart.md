# Quick Start

This guide will get you up and running with unimorph in under 5 minutes.

## Download Your First Language

Let's start by downloading a language dataset. We'll use Hebrew (`heb`) as an example:

```bash
unimorph download heb
```

You'll see output like:

```
Downloading heb...
Downloaded 33177 entries for heb
```

## Look Up Inflections

Now let's look up all the forms of a Hebrew verb. The `inflect` command takes a lemma (dictionary form) and shows all its inflected forms:

```bash
unimorph inflect -l heb כתב
```

Output:

```
LEMMA        FORM         FEATURES
------------------------------------------------------------
כתב         אכתוב       V;1;SG;FUT
כתב         יכתבו       V;3;PL;FUT;MASC
כתב         יכתוב       V;3;SG;FUT;MASC
כתב         כותב        V;SG;PRS;MASC
כתב         כתב         V;3;SG;PST;MASC
...

29 form(s) found.
```

## Analyze a Surface Form

What if you have a word and want to know what it is? Use `analyze`:

```bash
unimorph analyze -l heb כתבתי
```

Output:

```
FORM         LEMMA        FEATURES
------------------------------------------------------------
כתבתי       כתב         V;1;SG;PST

1 analysis(es) found.
```

## Search with Filters

Find entries matching specific criteria:

```bash
# Find all first person singular future forms
unimorph search -l heb --contains 1,SG,FUT --limit 5
```

```bash
# Find verbs (part of speech = V)
unimorph search -l heb --pos V --limit 5
```

```bash
# Search by lemma pattern (SQL LIKE wildcards)
unimorph search -l heb --lemma "כת%" --limit 5
```

## Check Dataset Statistics

```bash
unimorph stats heb
```

```
Statistics for heb:
  Total entries:    33177
  Unique lemmas:    1176
  Unique forms:     27286
  Unique features:  55
  Imported at:      2024-01-15 10:30:00 UTC
```

## Set a Default Language

Tired of typing `-l heb` every time? Set a default:

```bash
export UNIMORPH_LANG=heb
```

Or create a config file:

```bash
unimorph config init
```

Then edit `~/.config/unimorph/config.toml`:

```toml
default_lang = "heb"
```

Now you can just run:

```bash
unimorph inflect כתב
unimorph analyze כתבתי
```

## Output Formats

### JSON Output

Add `--json` for machine-readable output:

```bash
unimorph inflect -l heb כתב --json
```

### TSV for Piping

Use `--tsv` for tab-separated output without headers:

```bash
unimorph inflect -l heb כתב --tsv | head -5
```

```
כתב	אכתוב	V;1;SG;FUT
כתב	יכתבו	V;3;PL;FUT;MASC
כתב	יכתוב	V;3;SG;FUT;MASC
כתב	כותב	V;SG;PRS;MASC
כתב	כותבות	V;PL;PRS;FEM
```

### Export Full Dataset

Export an entire language to a file:

```bash
unimorph export -l heb -o hebrew.tsv
unimorph export -l heb -o hebrew.jsonl --format jsonl
```

Or to stdout for piping:

```bash
unimorph export -l heb -o - | grep "FUT" | wc -l
```

## Next Steps

- Browse [available languages](./unimorph/languages.md)
- Learn about the [feature schema](./unimorph/schema.md)
- Explore the full [CLI reference](./cli/overview.md)
- Use the [Rust library](./library/overview.md) in your projects
