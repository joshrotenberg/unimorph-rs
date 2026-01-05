# UniMorph Storage Backend Benchmark Results

Benchmarks run on 2024-01-05.

## Dataset Sizes

| Language | Entries | SQLite | DuckDB | Parquet |
|----------|---------|--------|--------|---------|
| Italian (ita) | 509,574 | 209 MB | 100 MB | 1.1 MB |
| Finnish (fin) | 2,737,048 | 522 MB | 301 MB | 7.2 MB |
| German (deu) | 519,116 | 575 MB* | 438 MB* | 2.0 MB |
| Spanish (spa) | 1,196,224 | 704 MB* | 526 MB* | 4.6 MB |

*Cumulative size (all languages in same DB)

## Performance Summary (ops/sec, higher is better)

### Italian (509k entries)

| Operation | SQLite | DuckDB | Parquet |
|-----------|--------|--------|---------|
| lookup_by_lemma | **21,497** | 2,124 | 110 |
| lookup_by_form | **125,910** | 422 | 110 |
| stats | 2 | **113** | 70 |
| search_features | 1 | 3 | **6** |

### Finnish (2.7M entries)

| Operation | SQLite | DuckDB | Parquet |
|-----------|--------|--------|---------|
| lookup_by_lemma | **14,741** | 1,102 | 135 |
| lookup_by_form | **170,288** | 245 | 132 |
| stats | 1 | **42** | 10 |
| search_features | 1 | 1 | **2** |

### German (519k entries)

| Operation | SQLite | DuckDB | Parquet |
|-----------|--------|--------|---------|
| lookup_by_lemma | **76,134** | 1,690 | 167 |
| lookup_by_form | **163,283** | 479 | 167 |
| stats | 5 | **151** | 96 |
| search_features | 4 | 6 | **8** |

### Spanish (1.2M entries)

| Operation | SQLite | DuckDB | Parquet |
|-----------|--------|--------|---------|
| lookup_by_lemma | **16,234** | 533 | 119 |
| lookup_by_form | **192,678** | 448 | 121 |
| stats | 2 | **75** | 36 |
| search_features | 1 | 2 | **3** |

## Key Findings

### 1. SQLite dominates point lookups
- 10-100x faster than DuckDB for lemma/form lookups
- B-tree indexes on lemma and form columns pay off massively
- Consistent performance across dataset sizes

### 2. Parquet has best compression
- 50-200x smaller than SQLite
- 30-75x smaller than DuckDB
- Ideal for distribution and cold storage

### 3. DuckDB is middle ground
- Better at analytics (stats queries) than SQLite
- Columnar format helps with aggregations
- Not optimized for single-row lookups

### 4. Analytics operations are slow everywhere
- Stats queries scan entire datasets
- Feature search with wildcards requires full scans
- Consider pre-computing stats and caching

## Recommendations for unimorph-core

1. **Primary storage: SQLite**
   - Best performance for the primary use cases (inflect, analyze)
   - Mature, well-tested, single-file database
   - Good enough compression with page-level compression options

2. **Distribution format: Parquet**
   - Use for downloading/distributing datasets
   - Convert to SQLite on first load
   - Keep Parquet files as source of truth for updates

3. **Cache computed stats**
   - Pre-compute DatasetStats on load
   - Store in metadata table or separate file
   - Invalidate on dataset update

4. **Consider indexes for features**
   - Current feature search is slow (full scan with LIKE)
   - Could add FTS5 for feature search
   - Or pre-compute common feature combinations
