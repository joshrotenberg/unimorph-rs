//! SQLite-based storage for UniMorph data.
//!
//! The store provides high-performance lookups for morphological data using
//! SQLite with B-tree indexes on lemma and form columns.
//!
//! # Schema
//!
//! ```sql
//! CREATE TABLE entries (
//!     id INTEGER PRIMARY KEY,
//!     lang TEXT NOT NULL,
//!     lemma TEXT NOT NULL,
//!     form TEXT NOT NULL,
//!     features TEXT NOT NULL
//! );
//!
//! CREATE INDEX idx_lang_lemma ON entries(lang, lemma);
//! CREATE INDEX idx_lang_form ON entries(lang, form);
//!
//! CREATE TABLE meta (
//!     lang TEXT PRIMARY KEY,
//!     entry_count INTEGER,
//!     unique_lemmas INTEGER,
//!     unique_forms INTEGER,
//!     unique_features INTEGER,
//!     imported_at TEXT,
//!     source_url TEXT
//! );
//! ```

use std::path::Path;

use rusqlite::{Connection, params};
use tracing::{debug, instrument};

use crate::{DatasetStats, Entry, FeatureBundle, LangCode, Result};

/// SQLite-based store for UniMorph data.
///
/// All languages are stored in a single database file with a `lang` column
/// to distinguish them. This simplifies management and enables cross-language
/// queries.
pub struct Store {
    pub(crate) conn: Connection,
}

impl Store {
    /// Open or create a store at the given path.
    ///
    /// If the database doesn't exist, it will be created with the required schema.
    /// If it exists, the schema will be verified/upgraded as needed.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        store.set_pragmas()?;
        Ok(store)
    }

    /// Create an in-memory store (useful for testing).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        store.set_pragmas()?;
        Ok(store)
    }

    /// Set SQLite pragmas for optimal read-heavy workload.
    fn set_pragmas(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA mmap_size = 268435456;
            PRAGMA temp_store = MEMORY;
            ",
        )?;
        Ok(())
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY,
                lang TEXT NOT NULL,
                lemma TEXT NOT NULL,
                form TEXT NOT NULL,
                features TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_lang_lemma ON entries(lang, lemma);
            CREATE INDEX IF NOT EXISTS idx_lang_form ON entries(lang, form);

            CREATE TABLE IF NOT EXISTS meta (
                lang TEXT PRIMARY KEY,
                entry_count INTEGER NOT NULL,
                unique_lemmas INTEGER NOT NULL,
                unique_forms INTEGER NOT NULL,
                unique_features INTEGER NOT NULL,
                imported_at TEXT NOT NULL,
                source_url TEXT
            );
            ",
        )?;
        Ok(())
    }

    /// Import entries for a language, replacing any existing data.
    ///
    /// This computes and caches statistics in the `meta` table.
    #[instrument(level = "debug", skip(self, entries), fields(entry_count = entries.len()))]
    pub fn import(
        &mut self,
        lang: &LangCode,
        entries: &[Entry],
        source_url: Option<&str>,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing entries for this language
        tx.execute("DELETE FROM entries WHERE lang = ?", params![lang.as_str()])?;
        tx.execute("DELETE FROM meta WHERE lang = ?", params![lang.as_str()])?;

        // Insert new entries
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

        // Compute and store stats
        let stats = DatasetStats::from_entries(entries);
        let now = chrono_lite_now();

        tx.execute(
            "INSERT INTO meta (lang, entry_count, unique_lemmas, unique_forms, unique_features, imported_at, source_url)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                lang.as_str(),
                stats.total_entries as i64,
                stats.unique_lemmas as i64,
                stats.unique_forms as i64,
                stats.unique_features as i64,
                now,
                source_url,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Look up all inflected forms of a lemma (dictionary form).
    ///
    /// This is the "inflect" operation: given a lemma like "parlare",
    /// return all its forms (parlo, parli, parla, ...).
    #[instrument(level = "debug", skip(self))]
    pub fn inflect(&self, lang: &str, lemma: &str) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ? AND lemma = ?")?;

        let entries = stmt
            .query_map(params![lang, lemma], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(lemma, form, features)| {
                FeatureBundle::new(&features)
                    .ok()
                    .map(|fb| Entry::new(lemma, form, fb))
            })
            .collect();

        Ok(entries)
    }

    /// Look up all lemmas that produce a given surface form.
    ///
    /// This is the "analyze" operation: given a form like "parlo",
    /// return all possible analyses (parlare -> parlo, ...).
    #[instrument(level = "debug", skip(self))]
    pub fn analyze(&self, lang: &str, form: &str) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, form, features FROM entries WHERE lang = ? AND form = ?")?;

        let entries = stmt
            .query_map(params![lang, form], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(lemma, form, features)| {
                FeatureBundle::new(&features)
                    .ok()
                    .map(|fb| Entry::new(lemma, form, fb))
            })
            .collect();

        Ok(entries)
    }

    /// Get cached statistics for a language.
    ///
    /// Returns `None` if the language is not in the store.
    pub fn stats(&self, lang: &str) -> Result<Option<DatasetStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT entry_count, unique_lemmas, unique_forms, unique_features
             FROM meta WHERE lang = ?",
        )?;

        let result = stmt.query_row(params![lang], |row| {
            Ok(DatasetStats::new(
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1)? as usize,
                row.get::<_, i64>(2)? as usize,
                row.get::<_, i64>(3)? as usize,
            ))
        });

        match result {
            Ok(stats) => Ok(Some(stats)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all languages in the store.
    pub fn languages(&self) -> Result<Vec<LangCode>> {
        let mut stmt = self.conn.prepare("SELECT lang FROM meta ORDER BY lang")?;

        let langs = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|s| LangCode::new(&s).ok())
            .collect();

        Ok(langs)
    }

    /// Check if a language is in the store.
    pub fn has_language(&self, lang: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT 1 FROM meta WHERE lang = ?")?;
        let exists = stmt.exists(params![lang])?;
        Ok(exists)
    }

    /// Delete a language from the store.
    pub fn delete_language(&mut self, lang: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM entries WHERE lang = ?", params![lang])?;
        tx.execute("DELETE FROM meta WHERE lang = ?", params![lang])?;
        tx.commit()?;
        Ok(())
    }

    /// Search for entries matching a feature pattern.
    ///
    /// The pattern uses `;` as separator and `*` as wildcard.
    /// This is slower than lemma/form lookups as it requires scanning.
    ///
    /// For better performance on large datasets, consider using
    /// `inflect` or `analyze` first, then filtering the results.
    #[instrument(level = "debug", skip(self))]
    pub fn search_features(&self, lang: &str, pattern: &str) -> Result<Vec<Entry>> {
        debug!(pattern, "searching features with pattern");
        // Convert pattern to SQL LIKE pattern
        // V;IND;*;1;* -> V;IND;%;1;%
        let sql_pattern = pattern.replace('*', "%");

        let mut stmt = self.conn.prepare(
            "SELECT lemma, form, features FROM entries
             WHERE lang = ? AND features LIKE ?",
        )?;

        let entries: Vec<Entry> = stmt
            .query_map(params![lang, sql_pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(lemma, form, features)| {
                FeatureBundle::new(&features)
                    .ok()
                    .map(|fb| Entry::new(lemma, form, fb))
            })
            // Double-check with proper pattern matching (LIKE is approximate)
            .filter(|e| e.features.matches_pattern(pattern))
            .collect();

        Ok(entries)
    }

    /// Get the import timestamp for a language.
    pub fn imported_at(&self, lang: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT imported_at FROM meta WHERE lang = ?")?;

        let result = stmt.query_row(params![lang], |row| row.get::<_, String>(0));

        match result {
            Ok(ts) => Ok(Some(ts)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

/// Simple timestamp without pulling in chrono.
fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    // Convert to ISO 8601-ish format
    let secs = duration.as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<Entry> {
        vec![
            Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
            Entry::parse_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
            Entry::parse_line("parlare\tparla\tV;IND;PRS;3;SG", 3).unwrap(),
            Entry::parse_line("essere\tsono\tV;IND;PRS;1;SG", 4).unwrap(),
            Entry::parse_line("essere\tsei\tV;IND;PRS;2;SG", 5).unwrap(),
            Entry::parse_line("essere\tè\tV;IND;PRS;3;SG", 6).unwrap(),
        ]
    }

    #[test]
    fn open_in_memory() {
        let store = Store::in_memory().unwrap();
        assert!(store.languages().unwrap().is_empty());
    }

    #[test]
    fn import_and_stats() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();
        let entries = sample_entries();

        store.import(&lang, &entries, None).unwrap();

        let stats = store.stats("ita").unwrap().unwrap();
        assert_eq!(stats.total_entries, 6);
        assert_eq!(stats.unique_lemmas, 2);
        assert_eq!(stats.unique_forms, 6);
    }

    #[test]
    fn inflect() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();
        store.import(&lang, &sample_entries(), None).unwrap();

        let forms = store.inflect("ita", "parlare").unwrap();
        assert_eq!(forms.len(), 3);
        assert!(forms.iter().any(|e| e.form == "parlo"));
        assert!(forms.iter().any(|e| e.form == "parli"));
        assert!(forms.iter().any(|e| e.form == "parla"));
    }

    #[test]
    fn analyze() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();
        store.import(&lang, &sample_entries(), None).unwrap();

        let analyses = store.analyze("ita", "sono").unwrap();
        assert_eq!(analyses.len(), 1);
        assert_eq!(analyses[0].lemma, "essere");
    }

    #[test]
    fn languages() {
        let mut store = Store::in_memory().unwrap();

        let ita: LangCode = "ita".parse().unwrap();
        let deu: LangCode = "deu".parse().unwrap();

        store.import(&ita, &sample_entries(), None).unwrap();
        store.import(&deu, &[], None).unwrap();

        let langs = store.languages().unwrap();
        assert_eq!(langs.len(), 2);
        assert!(langs.iter().any(|l| l.as_str() == "deu"));
        assert!(langs.iter().any(|l| l.as_str() == "ita"));
    }

    #[test]
    fn has_language() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();

        assert!(!store.has_language("ita").unwrap());
        store.import(&lang, &sample_entries(), None).unwrap();
        assert!(store.has_language("ita").unwrap());
    }

    #[test]
    fn delete_language() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();

        store.import(&lang, &sample_entries(), None).unwrap();
        assert!(store.has_language("ita").unwrap());

        store.delete_language("ita").unwrap();
        assert!(!store.has_language("ita").unwrap());
    }

    #[test]
    fn search_features() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();
        store.import(&lang, &sample_entries(), None).unwrap();

        // Search for first person singular verbs
        let results = store.search_features("ita", "V;IND;PRS;1;SG").unwrap();
        assert_eq!(results.len(), 2); // parlo, sono

        // Search with wildcard
        let results = store.search_features("ita", "V;IND;PRS;*;SG").unwrap();
        assert_eq!(results.len(), 6); // all entries
    }

    #[test]
    fn reimport_replaces_data() {
        let mut store = Store::in_memory().unwrap();
        let lang: LangCode = "ita".parse().unwrap();

        store.import(&lang, &sample_entries(), None).unwrap();
        assert_eq!(store.stats("ita").unwrap().unwrap().total_entries, 6);

        // Import fewer entries
        let fewer = vec![Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap()];
        store.import(&lang, &fewer, None).unwrap();
        assert_eq!(store.stats("ita").unwrap().unwrap().total_entries, 1);
    }
}
