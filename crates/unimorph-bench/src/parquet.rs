//! Parquet + Polars storage backend.

use std::collections::HashMap;
use std::path::PathBuf;

use polars::prelude::*;

use crate::{DatasetStats, Entry, Error, FeatureBundle, LangCode, Result, Store};

/// Parquet-based storage backend using Polars for queries.
///
/// Uses one Parquet file per language, stored in a cache directory.
/// Queries use Polars lazy frames with predicate pushdown.
pub struct ParquetStore {
    cache_dir: PathBuf,
    /// Cache of loaded DataFrames (lazy, loaded on demand)
    loaded: HashMap<LangCode, DataFrame>,
}

impl ParquetStore {
    /// Create a new Parquet store with the given cache directory.
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            cache_dir,
            loaded: HashMap::new(),
        })
    }

    /// Create an in-memory Parquet store (for testing).
    ///
    /// Uses a temporary directory that will be cleaned up.
    pub fn in_memory() -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir =
            std::env::temp_dir().join(format!("unimorph-bench-{}-{}", std::process::id(), id));
        Self::new(temp_dir)
    }

    /// Get the path to the Parquet file for a language.
    fn parquet_path(&self, lang: &LangCode) -> PathBuf {
        self.cache_dir.join(format!("{}.parquet", lang.as_str()))
    }

    /// Load a language's DataFrame, caching it.
    fn load_lang(&mut self, lang: &LangCode) -> Result<&DataFrame> {
        if !self.loaded.contains_key(lang) {
            let path = self.parquet_path(lang);
            if !path.exists() {
                return Err(Error::DatasetNotFound(lang.clone()));
            }
            let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?.collect()?;
            self.loaded.insert(lang.clone(), df);
        }
        Ok(self.loaded.get(lang).unwrap())
    }

    /// Convert a DataFrame row to an Entry.
    fn row_to_entry(lemma: &str, form: &str, features_str: &str) -> Result<Entry> {
        let features = FeatureBundle::new(features_str)?;
        Ok(Entry::new(lemma.to_string(), form.to_string(), features))
    }

    /// Get the number of entries for a language.
    pub fn count(&mut self, lang: &LangCode) -> Result<usize> {
        let df = self.load_lang(lang)?;
        Ok(df.height())
    }
}

impl Store for ParquetStore {
    fn init(&mut self, lang: &LangCode, entries: &[Entry]) -> Result<()> {
        // Build DataFrame from entries
        let lemmas: Vec<&str> = entries.iter().map(|e| e.lemma.as_str()).collect();
        let forms: Vec<&str> = entries.iter().map(|e| e.form.as_str()).collect();
        let features: Vec<&str> = entries.iter().map(|e| e.features.as_str()).collect();

        let df = DataFrame::new(vec![
            Column::new("lemma".into(), lemmas),
            Column::new("form".into(), forms),
            Column::new("features".into(), features),
        ])?;

        // Write to Parquet
        let path = self.parquet_path(lang);
        let file = std::fs::File::create(&path)?;
        ParquetWriter::new(file).finish(&mut df.clone())?;

        // Cache it
        self.loaded.insert(lang.clone(), df);

        Ok(())
    }

    fn lookup_by_lemma(&self, lang: &LangCode, lemma: &str) -> Result<Vec<Entry>> {
        let path = self.parquet_path(lang);
        if !path.exists() {
            return Err(Error::DatasetNotFound(lang.clone()));
        }

        // Use lazy frame with predicate pushdown
        let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?
            .filter(col("lemma").eq(lit(lemma)))
            .collect()?;

        Self::df_to_entries(&df)
    }

    fn lookup_by_form(&self, lang: &LangCode, form: &str) -> Result<Vec<Entry>> {
        let path = self.parquet_path(lang);
        if !path.exists() {
            return Err(Error::DatasetNotFound(lang.clone()));
        }

        let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?
            .filter(col("form").eq(lit(form)))
            .collect()?;

        Self::df_to_entries(&df)
    }

    fn search_features(&self, lang: &LangCode, pattern: &str) -> Result<Vec<Entry>> {
        let path = self.parquet_path(lang);
        if !path.exists() {
            return Err(Error::DatasetNotFound(lang.clone()));
        }

        // Load all entries and filter in Rust (pattern matching is complex)
        let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?.collect()?;

        let entries = Self::df_to_entries(&df)?;

        Ok(entries
            .into_iter()
            .filter(|e| e.features.matches_pattern(pattern))
            .collect())
    }

    fn stats(&self, lang: &LangCode) -> Result<DatasetStats> {
        let path = self.parquet_path(lang);
        if !path.exists() {
            return Err(Error::DatasetNotFound(lang.clone()));
        }

        let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?
            .select([
                len().alias("total"),
                col("lemma").n_unique().alias("unique_lemmas"),
                col("form").n_unique().alias("unique_forms"),
                col("features").n_unique().alias("unique_features"),
            ])
            .collect()?;

        let total = df.column("total")?.u32()?.get(0).unwrap_or(0) as usize;
        let unique_lemmas = df.column("unique_lemmas")?.u32()?.get(0).unwrap_or(0) as usize;
        let unique_forms = df.column("unique_forms")?.u32()?.get(0).unwrap_or(0) as usize;
        let unique_features = df.column("unique_features")?.u32()?.get(0).unwrap_or(0) as usize;

        Ok(DatasetStats {
            total_entries: total,
            unique_lemmas,
            unique_forms,
            unique_features,
        })
    }

    fn cross_lang_feature_count(&self, feature: &str) -> Result<HashMap<LangCode, usize>> {
        let mut counts = HashMap::new();

        // Scan all parquet files in cache directory
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "parquet")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(lang) = LangCode::new(stem)
            {
                let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())?.collect()?;

                let entries = Self::df_to_entries(&df)?;
                let count = entries
                    .iter()
                    .filter(|e| e.features.contains(feature))
                    .count();

                if count > 0 {
                    counts.insert(lang, count);
                }
            }
        }

        Ok(counts)
    }

    fn languages(&self) -> Result<Vec<LangCode>> {
        let mut langs = Vec::new();

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "parquet")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(lang) = LangCode::new(stem)
            {
                langs.push(lang);
            }
        }

        Ok(langs)
    }
}

impl ParquetStore {
    /// Convert a DataFrame to a Vec<Entry>.
    fn df_to_entries(df: &DataFrame) -> Result<Vec<Entry>> {
        let lemmas = df.column("lemma")?.str()?;
        let forms = df.column("form")?.str()?;
        let features = df.column("features")?.str()?;

        let mut entries = Vec::with_capacity(df.height());

        for i in 0..df.height() {
            let lemma = lemmas.get(i).ok_or_else(|| Error::MalformedEntry {
                line: i,
                reason: "missing lemma".to_string(),
            })?;
            let form = forms.get(i).ok_or_else(|| Error::MalformedEntry {
                line: i,
                reason: "missing form".to_string(),
            })?;
            let features_str = features.get(i).ok_or_else(|| Error::MalformedEntry {
                line: i,
                reason: "missing features".to_string(),
            })?;

            entries.push(Self::row_to_entry(lemma, form, features_str)?);
        }

        Ok(entries)
    }
}

impl Drop for ParquetStore {
    fn drop(&mut self) {
        // Clean up temp directory if it looks like our temp dir
        if self
            .cache_dir
            .to_str()
            .is_some_and(|s| s.contains("unimorph-bench-"))
        {
            let _ = std::fs::remove_dir_all(&self.cache_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<Entry> {
        vec![
            Entry::parse_tsv_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
            Entry::parse_tsv_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
            Entry::parse_tsv_line("parlare\tparla\tV;IND;PRS;3;SG", 3).unwrap(),
            Entry::parse_tsv_line("essere\tsono\tV;IND;PRS;1;SG", 4).unwrap(),
            Entry::parse_tsv_line("essere\tsei\tV;IND;PRS;2;SG", 5).unwrap(),
        ]
    }

    #[test]
    fn init_and_count() {
        let mut store = ParquetStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();
        assert_eq!(store.count(&lang).unwrap(), 5);
    }

    #[test]
    fn lookup_by_lemma() {
        let mut store = ParquetStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.lookup_by_lemma(&lang, "parlare").unwrap();
        assert_eq!(results.len(), 3);

        let results = store.lookup_by_lemma(&lang, "essere").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.lookup_by_lemma(&lang, "nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn lookup_by_form() {
        let mut store = ParquetStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.lookup_by_form(&lang, "parlo").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lemma, "parlare");
    }

    #[test]
    fn search_features() {
        let mut store = ParquetStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.search_features(&lang, "V;IND;PRS;1;SG").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search_features(&lang, "V;IND;PRS;*;SG").unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn stats() {
        let mut store = ParquetStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let stats = store.stats(&lang).unwrap();
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.unique_lemmas, 2);
        assert_eq!(stats.unique_forms, 5);
    }

    #[test]
    fn multiple_languages() {
        let mut store = ParquetStore::in_memory().unwrap();
        let ita = LangCode::new("ita").unwrap();
        let spa = LangCode::new("spa").unwrap();

        store.init(&ita, &sample_entries()).unwrap();
        store
            .init(
                &spa,
                &[Entry::parse_tsv_line("hablar\thablo\tV;IND;PRS;1;SG", 1).unwrap()],
            )
            .unwrap();

        let langs = store.languages().unwrap();
        assert_eq!(langs.len(), 2);
    }
}
