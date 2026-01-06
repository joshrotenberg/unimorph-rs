# About UniMorph

[UniMorph](https://unimorph.github.io/) is a collaborative project that provides morphological paradigms for the world's languages in a standardized format.

## What is Morphology?

Morphology is the study of word structure and how words change form to express different grammatical meanings. For example:

- **English**: "walk" -> "walks", "walked", "walking"
- **Spanish**: "hablar" -> "hablo", "hablas", "habla", "hablamos"...
- **Hebrew**: "כתב" -> "כותב", "כתבתי", "יכתוב"...

## What UniMorph Provides

UniMorph datasets contain mappings from **lemmas** (dictionary forms) to their **inflected forms**, along with **morphological features** describing each form.

### Data Format

Each entry is a triple:

```
lemma <TAB> form <TAB> features
```

Example (Italian):
```
parlare	parlo	V;IND;PRS;1;SG
parlare	parli	V;IND;PRS;2;SG
parlare	parla	V;IND;PRS;3;SG
parlare	parliamo	V;IND;PRS;1;PL
parlare	parlate	V;IND;PRS;2;PL
parlare	parlano	V;IND;PRS;3;PL
```

### Coverage

UniMorph includes data for 100+ languages, ranging from:

- **High-resource languages**: English, Spanish, German, French
- **Medium-resource languages**: Finnish, Hungarian, Turkish
- **Low-resource languages**: Many endangered and under-documented languages

## Data Sources

UniMorph data comes from:

- Wiktionary extractions
- Linguistic databases
- Academic contributions
- Community submissions

## Use Cases

### Natural Language Processing

- Training morphological inflection models
- Data augmentation for NLU systems
- Lemmatization and stemming lookup tables

### Language Learning

- Conjugation practice applications
- Flashcard generation
- Grammar reference tools

### Linguistic Research

- Cross-linguistic typology studies
- Morphological complexity analysis
- Paradigm structure research

### Lexicography

- Dictionary development
- Inflection table generation
- Coverage verification

## The UniMorph Schema

UniMorph uses a standardized feature schema across all languages, making cross-linguistic comparison possible. Features are organized into dimensions:

- Part of Speech (V, N, ADJ, ...)
- Person (1, 2, 3)
- Number (SG, PL, DU)
- Tense (PST, PRS, FUT)
- And many more...

See [Feature Schema](./schema.md) for the complete schema.

## Contributing to UniMorph

UniMorph is open source. Each language has its own GitHub repository:

- Main site: [unimorph.github.io](https://unimorph.github.io/)
- Organization: [github.com/unimorph](https://github.com/unimorph)

Contributions welcome:

- Report data errors
- Add missing forms
- Contribute new languages

## Citation

If you use UniMorph in research, please cite:

```bibtex
@inproceedings{mccarthy-etal-2020-unimorph,
    title = "{U}ni{M}orph 3.0: Universal Morphology",
    author = "McCarthy, Arya D. and others",
    booktitle = "LREC",
    year = "2020",
}
```

## Related Projects

- **SIGMORPHON**: Shared tasks on morphological analysis
- **Universal Dependencies**: Syntactic annotation
- **Lexical Markup Framework**: ISO standard for lexical resources
