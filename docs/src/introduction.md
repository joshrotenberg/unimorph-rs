# Introduction

**unimorph-rs** is a complete Rust toolkit for working with [UniMorph](https://unimorph.github.io/) morphological data. It provides both a command-line interface and a Rust library for downloading, querying, and analyzing morphological inflection data across 100+ languages.

## What is UniMorph?

UniMorph is a collaborative project providing morphological paradigms for the world's languages. Each language dataset contains entries mapping lemmas (dictionary forms) to their inflected forms along with morphological feature annotations.

For example, in Italian:

| Lemma | Form | Features |
|-------|------|----------|
| parlare | parlo | V;IND;PRS;1;SG |
| parlare | parli | V;IND;PRS;2;SG |
| parlare | parla | V;IND;PRS;3;SG |
| parlare | parliamo | V;IND;PRS;1;PL |

## Features

- **Fast lookups**: SQLite-backed storage with indexed queries
- **100+ languages**: Access to all UniMorph language datasets
- **Flexible querying**: Search by lemma, form, features, or part of speech
- **Multiple output formats**: Table, JSON, TSV for scripting
- **Pipe-friendly**: Output designed for Unix pipelines
- **Offline-first**: Data cached locally after download
- **Library + CLI**: Use as a Rust library or command-line tool

## Use Cases

- **Language learners**: Look up conjugations and declensions
- **NLP researchers**: Training data for morphological models
- **Lexicographers**: Verify inflection paradigms
- **Educators**: Build conjugation practice tools
- **Linguists**: Cross-linguistic morphological analysis

## Quick Example

```bash
# Download Hebrew dataset
unimorph download heb

# Look up all forms of a verb
unimorph inflect -l heb כתב

# Analyze a surface form
unimorph analyze -l heb כתבתי

# Search for plural masculine forms
unimorph search -l heb --contains PL,MASC --limit 10
```

## Getting Started

Head to the [Installation](./installation.md) guide to get started, or jump straight to the [Quick Start](./quickstart.md) for a hands-on introduction.
