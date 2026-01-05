//! SQLite storage backend.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, params};

use crate::{DatasetStats, Entry, FeatureBundle, LangCode, Result, Store};

/// SQLite-based storage backend.
///
/// Uses a single database file with a `lang` column to support
/// multiple languages. Indexes on `(lang, lemma)` and `(lang, form)`
/// provide fast point lookups.
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Create a new in-memory SQLite store.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    /// Open or create a SQLite store at the given path.
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
                lang TEXT NOT NULL,
                lemma TEXT NOT NULL,
                form TEXT NOT NULL,
                features TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_lang_lemma ON entries(lang, lemma);
            CREATE INDEX IF NOT EXISTS idx_lang_form ON entries(lang, form);
            ",
        )?;
        Ok(())
    }

    /// Get the number of entries for a language.
    pub fn count(&self, lang: &LangCode) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE lang = ?",
            params![lang.as_str()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

impl Store for SqliteStore {
    fn init(&mut self, lang: &LangCode, entries: &[Entry]) -> Result<()> {
        let tx = self.conn.transaction()?;

        {
            let mut stmt = tx
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

        tx.commit()?;
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
        // For now, we scan all entries and filter in Rust.
        // This could be optimized with FTS5 or a normalized features table.
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
        let total_entries: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE lang = ?",
            params![lang.as_str()],
            |row| row.get(0),
        )?;

        let unique_lemmas: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT lemma) FROM entries WHERE lang = ?",
            params![lang.as_str()],
            |row| row.get(0),
        )?;

        let unique_forms: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT form) FROM entries WHERE lang = ?",
            params![lang.as_str()],
            |row| row.get(0),
        )?;

        let unique_features: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT features) FROM entries WHERE lang = ?",
            params![lang.as_str()],
            |row| row.get(0),
        )?;

        Ok(DatasetStats {
            total_entries: total_entries as usize,
            unique_lemmas: unique_lemmas as usize,
            unique_forms: unique_forms as usize,
            unique_features: unique_features as usize,
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
        let mut store = SqliteStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();
        assert_eq!(store.count(&lang).unwrap(), 5);
    }

    #[test]
    fn lookup_by_lemma() {
        let mut store = SqliteStore::in_memory().unwrap();
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
        let mut store = SqliteStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let results = store.lookup_by_form(&lang, "parlo").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lemma, "parlare");

        let results = store.lookup_by_form(&lang, "sono").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lemma, "essere");
    }

    #[test]
    fn search_features() {
        let mut store = SqliteStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        // All 1st person singular
        let results = store.search_features(&lang, "V;IND;PRS;1;SG").unwrap();
        assert_eq!(results.len(), 2);

        // All singular with wildcard
        let results = store.search_features(&lang, "V;IND;PRS;*;SG").unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn stats() {
        let mut store = SqliteStore::in_memory().unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.init(&lang, &sample_entries()).unwrap();

        let stats = store.stats(&lang).unwrap();
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.unique_lemmas, 2);
        assert_eq!(stats.unique_forms, 5);
    }

    #[test]
    fn multiple_languages() {
        let mut store = SqliteStore::in_memory().unwrap();
        let ita = LangCode::new("ita").unwrap();
        let spa = LangCode::new("spa").unwrap();

        store.init(&ita, &sample_entries()).unwrap();
        store
            .init(
                &spa,
                &[Entry::parse_tsv_line("hablar\thablo\tV;IND;PRS;1;SG", 1).unwrap()],
            )
            .unwrap();

        assert_eq!(store.count(&ita).unwrap(), 5);
        assert_eq!(store.count(&spa).unwrap(), 1);

        let langs = store.languages().unwrap();
        assert_eq!(langs.len(), 2);
    }
}
