# Feature Schema

UniMorph uses a standardized feature schema to annotate morphological forms. Features are semicolon-separated and position-dependent within each language.

## Feature Format

```
FEATURE1;FEATURE2;FEATURE3;...
```

Example: `V;IND;PRS;1;SG` means:
- V = Verb
- IND = Indicative mood
- PRS = Present tense
- 1 = First person
- SG = Singular number

## Feature Dimensions

### Part of Speech

| Feature | Description |
|---------|-------------|
| `V` | Verb |
| `N` | Noun |
| `ADJ` | Adjective |
| `ADV` | Adverb |
| `PRO` | Pronoun |
| `DET` | Determiner |
| `ADP` | Adposition |
| `NUM` | Numeral |
| `CONJ` | Conjunction |
| `PART` | Particle |
| `INTJ` | Interjection |
| `V.MSDR` | Verbal noun / Masdar |
| `V.PTCP` | Participle |
| `V.CVB` | Converb |

### Person

| Feature | Description |
|---------|-------------|
| `1` | First person |
| `2` | Second person |
| `3` | Third person |
| `4` | Fourth person (obviate) |
| `INCL` | Inclusive |
| `EXCL` | Exclusive |

### Number

| Feature | Description |
|---------|-------------|
| `SG` | Singular |
| `PL` | Plural |
| `DU` | Dual |
| `TRI` | Trial |
| `PAUC` | Paucal |
| `GRPL` | Greater plural |

### Gender

| Feature | Description |
|---------|-------------|
| `MASC` | Masculine |
| `FEM` | Feminine |
| `NEUT` | Neuter |
| `NAKH` | Animate (Algonquian) |

### Case

| Feature | Description |
|---------|-------------|
| `NOM` | Nominative |
| `ACC` | Accusative |
| `GEN` | Genitive |
| `DAT` | Dative |
| `INS` | Instrumental |
| `LOC` | Locative |
| `ABL` | Ablative |
| `VOC` | Vocative |
| `ESS` | Essive |
| `TRANS` | Translative |
| `COM` | Comitative |
| `PRIV` | Privative |
| `PRT` | Partitive |
| And many more... | |

### Tense

| Feature | Description |
|---------|-------------|
| `PRS` | Present |
| `PST` | Past |
| `FUT` | Future |
| `IPFV` | Imperfective |
| `PFV` | Perfective |
| `PRF` | Perfect |
| `PLPRF` | Pluperfect |
| `PROSP` | Prospective |

### Aspect

| Feature | Description |
|---------|-------------|
| `IPFV` | Imperfective |
| `PFV` | Perfective |
| `HAB` | Habitual |
| `PROG` | Progressive |
| `ITER` | Iterative |

### Mood

| Feature | Description |
|---------|-------------|
| `IND` | Indicative |
| `SBJV` | Subjunctive |
| `IMP` | Imperative |
| `COND` | Conditional |
| `OPT` | Optative |
| `POT` | Potential |
| `PURP` | Purposive |

### Voice

| Feature | Description |
|---------|-------------|
| `ACT` | Active |
| `PASS` | Passive |
| `MID` | Middle |
| `ANTIP` | Antipassive |
| `CAUS` | Causative |

### Finiteness

| Feature | Description |
|---------|-------------|
| `FIN` | Finite |
| `NFIN` | Non-finite |

### Definiteness

| Feature | Description |
|---------|-------------|
| `DEF` | Definite |
| `NDEF` | Indefinite |
| `SPEC` | Specific |
| `NSPEC` | Non-specific |

### Comparison

| Feature | Description |
|---------|-------------|
| `CMPR` | Comparative |
| `SPRL` | Superlative |

### Polarity

| Feature | Description |
|---------|-------------|
| `POS` | Positive |
| `NEG` | Negative |

### Possession

| Feature | Description |
|---------|-------------|
| `PSS1S` | 1st person singular possessor |
| `PSS2S` | 2nd person singular possessor |
| `PSS3S` | 3rd person singular possessor |
| `PSS1P` | 1st person plural possessor |
| `PSS2P` | 2nd person plural possessor |
| `PSS3P` | 3rd person plural possessor |
| `PSSD` | Possessed form |

## Language-Specific Features

Some languages have additional features not listed above. Use `unimorph features -l <lang> --list` to see all features used in a specific language.

## Feature Position

Feature positions vary by language. For example:

**Hebrew verbs**: `V;PERSON;NUMBER;TENSE;GENDER`
```
V;1;SG;PST     (1st person singular past)
V;3;PL;FUT;MASC (3rd person plural future masculine)
```

**Italian verbs**: `V;MOOD;TENSE;PERSON;NUMBER`
```
V;IND;PRS;1;SG  (indicative present 1st singular)
V;SBJV;PST;3;PL (subjunctive past 3rd plural)
```

## Working with Features

### CLI

```bash
# List all features in a language
unimorph features -l heb --list

# See feature statistics
unimorph features -l heb --stats

# Find entries with a feature
unimorph features -l heb --search FUT

# Search by feature pattern
unimorph search -l heb -f "V;1;SG;*"

# Search by contained features
unimorph search -l heb --contains PL,MASC
```

### Library

```rust
use unimorph_core::FeatureBundle;

let features: FeatureBundle = "V;1;SG;PST".parse()?;

// Check for specific feature
if features.contains("PST") {
    println!("Past tense");
}

// Pattern matching
if features.matches("V;*;SG;*") {
    println!("Singular verb");
}
```

## References

- [UniMorph Schema Documentation](https://unimorph.github.io/doc/unimorph-schema.pdf)
- [Leipzig Glossing Rules](https://www.eva.mpg.de/lingua/resources/glossing-rules.php)
