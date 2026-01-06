# export

Export a language dataset to file.

**Alias:** `x`

## Synopsis

```
unimorph export [OPTIONS]
```

## Description

Exports a downloaded language dataset to a file in TSV or JSONL format. Useful for integrating with other tools, creating backups, or processing data with external programs.

## Options

| Option | Description |
|--------|-------------|
| `-l, --lang <LANG>` | Language code (ISO 639-3) |
| `-o, --output <PATH>` | Output file path (use `-` for stdout) |
| `-F, --format <FORMAT>` | Output format: `tsv` or `jsonl` (auto-detected from extension) |

## Examples

### Export to TSV

```bash
unimorph export -l heb -o hebrew.tsv
```

```
Exported 33177 entries to hebrew.tsv
```

### Export to JSONL

```bash
unimorph export -l heb -o hebrew.jsonl
```

Or explicitly specify format:

```bash
unimorph export -l heb -o hebrew.json --format jsonl
```

### Export to Stdout

Use `-o -` to write to stdout:

```bash
unimorph export -l heb -o - --format tsv | head -5
```

```
איבד	אאבד	V;1;SG;FUT
איבזר	אאבזר	V;1;SG;FUT
איבטח	אאבטח	V;1;SG;FUT
האביס	אאביס	V;1;SG;FUT
אבל	אאבל	V;1;SG;FUT
```

The status message goes to stderr, so piping works correctly:

```bash
unimorph export -l heb -o - 2>/dev/null | wc -l
33177
```

### Scripting Examples

```bash
# Filter exported data
unimorph export -l heb -o - | grep "FUT" > future_forms.tsv

# Export and compress
unimorph export -l heb -o - | gzip > hebrew.tsv.gz

# Export multiple languages
for lang in heb ita deu; do
  unimorph export -l "$lang" -o "${lang}.tsv"
done

# Convert to CSV
unimorph export -l heb -o - | tr '\t' ',' > hebrew.csv
```

## Output Formats

### TSV (Tab-Separated Values)

```
lemma<TAB>form<TAB>features
```

Example:
```
hablar	hablo	V;IND;PRS;1;SG
hablar	hablas	V;IND;PRS;2;SG
```

### JSONL (JSON Lines)

One JSON object per line:

```json
{"lemma":"hablar","form":"hablo","features":"V;IND;PRS;1;SG"}
{"lemma":"hablar","form":"hablas","features":"V;IND;PRS;2;SG"}
```

## Notes

- Format is auto-detected from file extension (`.tsv` or `.jsonl`)
- Use `--format` to override auto-detection
- Stdout export writes status to stderr to avoid polluting data

## See Also

- [search](./search.md) - Export filtered subsets with `--tsv`
- [download](./download.md) - Download datasets
