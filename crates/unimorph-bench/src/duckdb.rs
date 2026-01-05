//! DuckDB storage backend.

use std::collections::HashMap;
use std::path::Path;

use duckdb::{Connection, params};

use crate::{DatasetStats, Entry, FeatureBundle, LangCode, Result, Store};

/// DuckDB-based storage backend.
///
/// Uses a single database file with a `lang` column. DuckDB is columnar
/// and optimized for analytics, but also supports indexes for point lookups.
pub struct DuckDbStore {
    conn: Connection,
}

impl DuckDbStore {
    /// Create a new in-memory DuckDB store.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    /// Open or create a DuckDB store at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let mut store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    fn create_schema(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS entries (
                lang VARCHAR NOT NULL,
                lemma VARCHAR NOT NULL,
                form VARCHAR NOT NULL,
                features VARCHAR NOT NULL
            );
            ",
        )?;

        // DuckDB supports indexes but they work differently than SQLite.
        // We create them after initial data load for better performance.
        Ok(())
    }

    /// Create indexes after data has been loaded.
    ///
    /// DuckDB indexes are more effective when created after bulk inserts.
    pub fn create_indexes(&mut self) -> Result<()> {
        // DuckDB uses ART indexes by default
        self.conn.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_lang_lemma ON entries(lang, lemma);
            CREATE INDEX IF NOT EXISTS idx_lang_form ON entries(lang, form);
            ",
        )?;
        Ok(())
    }

    /// Get the number of entries for a language.
    pub fn count(&self, lang: &LangCode) -> Result<usize> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM entries WHERE lang = ?")?;
        let count: i64 = stmt.query_row(params![lang.as_str()], |row| row.get(0))?;
        Ok(count as usize)
    }
}

impl Store for DuckDbStore {
    fn init(&mut self, lang: &LangCode, entries: &[Entry]) -> Result<()> {
        // DuckDB performs better with prepared statements in a transaction
        self.conn.execute_batch("BEGIN TRANSACTION")?;

        {
            let mut stmt = self
                .conn
                .prepare("INSERT INTO entries (lang, lemma, form, features) VALUES (?, ?, ?, ?)")?;

            for entry in entries {
                stmt.execute(params![
                    lang.as_str(),
                    &entry.lemma,
                    &entry.form,
                    entry.features.as_str(),
                ])?;
            }
        }

        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    fn lookup_by_lemma(&self, lang: &LangCode, lemma: &str) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ? AND lemma = ?")?;

        let entries = stmt
            .query_map(params![lang.as_str(), lemma], |row| {
                let lemma: String = row.get(0)?;
                let form: String = row.get(1)?;
                let features_str: String = row.get(2)?;
                Ok((lemma, form, features_str))
            })?
            .map(|r| {
                let (lemma, form, features_str) = r?;
                let features = FeatureBundle::new(&features_str)?;
                Ok(Entry::new(lemma, form, features))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    fn lookup_by_form(&self, lang: &LangCode, form: &str) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ? AND form = ?")?;

        let entries = stmt
            .query_map(params![lang.as_str(), form], |row| {
                let lemma: String = row.get(0)?;
                let form: String = row.get(1)?;
                let features_str: String = row.get(2)?;
                Ok((lemma, form, features_str))
            })?
            .map(|r| {
                let (lemma, form, features_str) = r?;
                let features = FeatureBundle::new(&features_str)?;
                Ok(Entry::new(lemma, form, features))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    fn search_features(&self, lang: &LangCode, pattern: &str) -> Result<Vec<Entry>> {
        // Scan and filter in Rust (same approach as SQLite for now)
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ?")?;

        let entries = stmt
            .query_map(params![lang.as_str()], |row| {
                let lemma: String = row.get(0)?;
                let form: String = row.get(1)?;
                let features_str: String = row.get(2)?;
                Ok((lemma, form, features_str))
            })?
            .filter_map(|r| {
                let (lemma, form, features_str) = r.ok()?;
                let features = FeatureBundle::new(&features_str).ok()?;
                if features.matches_pattern(pattern) {
                    Some(Ok(Entry::new(lemma, form, features)))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    fn stats(&self, lang: &LangCode) -> Result<DatasetStats> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                COUNT(*),
                COUNT(DISTINCT lemma),
                COUNT(DISTINCT form),
                COUNT(DISTINCT features)
            FROM entries
            WHERE lang = ?
            ",
        )?;

        let (total, lemmas, forms, features): (i64, i64, i64, i64) = stmt
            .query_row(params![lang.as_str()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;

        Ok(DatasetStats {
            total_entries: total as usize,
            unique_lemmas: lemmas as usize,
            unique_forms: forms as usize,
            unique_features: features as usize,
        })
    }

    fn cross_lang_feature_count(&self, feature: &str) -> Result<HashMap<LangCode, usize>> {
        let mut stmt = self.conn.prepare("SELECT lang, features FROM entries")?;

        let mut counts: HashMap<LangCode, usize> = HashMap::new();

        let rows = stmt.query_map([], |row| {
            let lang_str: String = row.get(0)?;
            let features_str: String = row.get(1)?;
            Ok((lang_str, features_str))
        })?;

        for row in rows {
            let (lang_str, features_str) = row?;
            if let Ok(features) = FeatureBundle::new(&features_str)
                && features.contains(feature)
                && let Ok(lang) = LangCode::new(&lang_str)
            {
                *counts.entry(lang).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }

    fn languages(&self) -> Result<Vec<LangCode>> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT lang FROM entries")?;

        let langs = stmt
            .query_map([], |row| {
                let lang_str: String = row.get(0)?;
                Ok(lang_str)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|s| LangCode::new(&s).ok())
            .collect();

        Ok(langs)
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
        let mut store = DuckDbStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();
        assert_eq!(store.count(&lang).unwrap(), 5);
    }

    #[test]
    fn lookup_by_lemma() {
        let mut store = DuckDbStore::in_memory().unwrap();
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
        let mut store = DuckDbStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.lookup_by_form(&lang, "parlo").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lemma, "parlare");
    }

    #[test]
    fn search_features() {
        let mut store = DuckDbStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.search_features(&lang, "V;IND;PRS;1;SG").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search_features(&lang, "V;IND;PRS;*;SG").unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn stats() {
        let mut store = DuckDbStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let stats = store.stats(&lang).unwrap();
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.unique_lemmas, 2);
        assert_eq!(stats.unique_forms, 5);
    }

    #[test]
    fn with_indexes() {
        let mut store = DuckDbStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();
        store.create_indexes().unwrap();

        // Queries should still work with indexes
        let results = store.lookup_by_lemma(&lang, "parlare").unwrap();
        assert_eq!(results.len(), 3);
    }
}
