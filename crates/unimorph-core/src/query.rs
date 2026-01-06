//! Fluent query builder for UniMorph data.
//!
//! The query builder provides an ergonomic API for constructing complex queries
//! without writing raw SQL or chaining multiple method calls.
//!
//! # Examples
//!
//! ```ignore
//! use unimorph_core::Store;
//!
//! let store = Store::open("datasets.db")?;
//!
//! // Find all indicative present forms of "parlare"
//! let forms = store.query("ita")
//!     .lemma("parlare")
//!     .features_contain(&["IND", "PRS"])
//!     .execute()?;
//!
//! // Find all first person singular verbs
//! let verbs = store.query("ita")
//!     .features_match("V;*;*;1;SG")
//!     .limit(100)
//!     .execute()?;
//!
//! // Reverse lookup with filtering
//! let analyses = store.query("ita")
//!     .form("sono")
//!     .features_contain(&["V"])
//!     .execute()?;
//! ```

use crate::{Entry, FeatureBundle, Result};
use rusqlite::Connection;
use tracing::{debug, instrument};

/// A fluent query builder for UniMorph entries.
///
/// Create a query builder using [`Store::query`](crate::Store::query).
#[derive(Debug, Clone)]
pub struct QueryBuilder<'a> {
    conn: &'a Connection,
    lang: String,
    lemma: Option<String>,
    form: Option<String>,
    features_pattern: Option<String>,
    features_contain: Vec<String>,
    pos: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new query builder for the given language.
    pub(crate) fn new(conn: &'a Connection, lang: &str) -> Self {
        Self {
            conn,
            lang: lang.to_string(),
            lemma: None,
            form: None,
            features_pattern: None,
            features_contain: Vec::new(),
            pos: None,
            limit: None,
            offset: None,
        }
    }

    /// Filter by lemma (dictionary form).
    ///
    /// Supports SQL LIKE wildcards: `%` matches any sequence of characters,
    /// `_` matches any single character. If no wildcards are present,
    /// performs an exact match.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Exact match
    /// let forms = store.query("ita").lemma("parlare").execute()?;
    ///
    /// // Wildcard match - all lemmas starting with "parl"
    /// let forms = store.query("ita").lemma("parl%").execute()?;
    /// ```
    pub fn lemma(mut self, lemma: &str) -> Self {
        self.lemma = Some(lemma.to_string());
        self
    }

    /// Filter by surface form.
    ///
    /// Supports SQL LIKE wildcards: `%` matches any sequence of characters,
    /// `_` matches any single character. If no wildcards are present,
    /// performs an exact match (useful for reverse lookups/analyze).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Exact match
    /// let analyses = store.query("ita").form("parlo").execute()?;
    ///
    /// // Wildcard match - all forms ending with "ando"
    /// let forms = store.query("ita").form("%ando").execute()?;
    /// ```
    pub fn form(mut self, form: &str) -> Self {
        self.form = Some(form.to_string());
        self
    }

    /// Filter by feature pattern with wildcards.
    ///
    /// The pattern uses `;` as separator and `*` as wildcard.
    /// The number of pattern elements must match the number of features.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // All first person singular verbs
    /// let verbs = store.query("ita")
    ///     .features_match("V;*;*;1;SG")
    ///     .execute()?;
    /// ```
    pub fn features_match(mut self, pattern: &str) -> Self {
        self.features_pattern = Some(pattern.to_string());
        self
    }

    /// Filter to entries containing all specified features (in any position).
    ///
    /// This is more flexible than `features_match` as it doesn't require
    /// knowing the exact feature positions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // All indicative present verbs (regardless of person/number)
    /// let verbs = store.query("ita")
    ///     .features_contain(&["V", "IND", "PRS"])
    ///     .execute()?;
    /// ```
    pub fn features_contain(mut self, features: &[&str]) -> Self {
        self.features_contain = features.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter by part of speech.
    ///
    /// Common POS tags: V (verb), N (noun), ADJ (adjective), ADV (adverb).
    /// This is a shorthand for `features_contain(&["V"])`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let nouns = store.query("ita").pos("N").execute()?;
    /// ```
    pub fn pos(mut self, pos: &str) -> Self {
        self.pos = Some(pos.to_string());
        self
    }

    /// Limit the number of results.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sample = store.query("ita").limit(10).execute()?;
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Skip the first N results (for pagination).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get results 11-20
    /// let page2 = store.query("ita")
    ///     .offset(10)
    ///     .limit(10)
    ///     .execute()?;
    /// ```
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Execute the query and return matching entries.
    #[instrument(level = "debug", skip(self), fields(lang = %self.lang))]
    pub fn execute(self) -> Result<Vec<Entry>> {
        debug!(
            lemma = ?self.lemma,
            form = ?self.form,
            features_pattern = ?self.features_pattern,
            features_contain = ?self.features_contain,
            pos = ?self.pos,
            limit = ?self.limit,
            offset = ?self.offset,
            "executing query"
        );
        let mut sql = String::from("SELECT lemma, form, features FROM entries WHERE lang = ?");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(self.lang.clone())];

        // Build WHERE clauses
        if let Some(ref lemma) = self.lemma {
            if lemma.contains('%') || lemma.contains('_') {
                sql.push_str(" AND lemma LIKE ?");
            } else {
                sql.push_str(" AND lemma = ?");
            }
            params_vec.push(Box::new(lemma.clone()));
        }

        if let Some(ref form) = self.form {
            if form.contains('%') || form.contains('_') {
                sql.push_str(" AND form LIKE ?");
            } else {
                sql.push_str(" AND form = ?");
            }
            params_vec.push(Box::new(form.clone()));
        }

        if let Some(ref pattern) = self.features_pattern {
            // Convert pattern to SQL LIKE: V;IND;*;1;* -> V;IND;%;1;%
            let sql_pattern = pattern.replace('*', "%");
            sql.push_str(" AND features LIKE ?");
            params_vec.push(Box::new(sql_pattern));
        }

        // Check if we have post-filters that need to run after SQL
        let has_post_filters = !self.features_contain.is_empty()
            || self.pos.is_some()
            || self.features_pattern.is_some();

        // Only apply SQL LIMIT/OFFSET if no post-filters (otherwise we filter after fetch)
        if !has_post_filters {
            if let Some(limit) = self.limit {
                sql.push_str(&format!(" LIMIT {}", limit));
                if let Some(offset) = self.offset {
                    sql.push_str(&format!(" OFFSET {}", offset));
                }
            } else if self.offset.is_some() {
                // OFFSET without LIMIT: use a very large limit
                sql.push_str(&format!(" LIMIT -1 OFFSET {}", self.offset.unwrap()));
            }
        }

        // Execute query
        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let iter = stmt
            .query_map(params_refs.as_slice(), |row| {
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
            // Apply post-filters that can't be done in SQL
            .filter(|entry| {
                // features_contain filter
                if !self.features_contain.is_empty() {
                    let refs: Vec<&str> =
                        self.features_contain.iter().map(|s| s.as_str()).collect();
                    if !entry.features.contains_all(&refs) {
                        return false;
                    }
                }
                // POS filter
                if let Some(ref pos) = self.pos
                    && !entry.features.contains(pos)
                {
                    return false;
                }
                // Pattern match (double-check SQL LIKE results)
                if let Some(ref pattern) = self.features_pattern
                    && !entry.features.matches_pattern(pattern)
                {
                    return false;
                }
                true
            });

        // Apply limit/offset after post-filters when post-filters are present
        let entries: Vec<Entry> = if has_post_filters {
            match (self.offset, self.limit) {
                (Some(offset), Some(limit)) => iter.skip(offset).take(limit).collect(),
                (Some(offset), None) => iter.skip(offset).collect(),
                (None, Some(limit)) => iter.take(limit).collect(),
                (None, None) => iter.collect(),
            }
        } else {
            iter.collect()
        };

        debug!(count = entries.len(), "query returned entries");
        Ok(entries)
    }

    /// Execute the query and return the count of matching entries.
    ///
    /// More efficient than `execute().len()` for large result sets.
    pub fn count(self) -> Result<usize> {
        // For count, we need to execute and count due to post-filters
        // If no post-filters, we could use SQL COUNT
        let has_post_filters = !self.features_contain.is_empty()
            || self.pos.is_some()
            || self.features_pattern.is_some();

        if has_post_filters {
            // Must fetch and filter
            Ok(self.execute()?.len())
        } else {
            // Can use SQL COUNT
            let mut sql = String::from("SELECT COUNT(*) FROM entries WHERE lang = ?");
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(self.lang.clone())];

            if let Some(ref lemma) = self.lemma {
                if lemma.contains('%') || lemma.contains('_') {
                    sql.push_str(" AND lemma LIKE ?");
                } else {
                    sql.push_str(" AND lemma = ?");
                }
                params_vec.push(Box::new(lemma.clone()));
            }

            if let Some(ref form) = self.form {
                if form.contains('%') || form.contains('_') {
                    sql.push_str(" AND form LIKE ?");
                } else {
                    sql.push_str(" AND form = ?");
                }
                params_vec.push(Box::new(form.clone()));
            }

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|b| b.as_ref()).collect();
            let count: i64 = self
                .conn
                .query_row(&sql, params_refs.as_slice(), |row| row.get(0))?;

            Ok(count as usize)
        }
    }

    /// Check if any entries match the query.
    ///
    /// More efficient than `execute().is_empty()`.
    pub fn exists(self) -> Result<bool> {
        Ok(!self.limit(1).execute()?.is_empty())
    }

    /// Get the first matching entry, if any.
    pub fn first(self) -> Result<Option<Entry>> {
        Ok(self.limit(1).execute()?.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use crate::Store;

    fn setup_store() -> Store {
        let mut store = Store::in_memory().unwrap();
        let lang = "ita".parse().unwrap();

        let entries = vec![
            crate::Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
            crate::Entry::parse_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
            crate::Entry::parse_line("parlare\tparla\tV;IND;PRS;3;SG", 3).unwrap(),
            crate::Entry::parse_line("parlare\tparliamo\tV;IND;PRS;1;PL", 4).unwrap(),
            crate::Entry::parse_line("parlare\tparlate\tV;IND;PRS;2;PL", 5).unwrap(),
            crate::Entry::parse_line("parlare\tparlano\tV;IND;PRS;3;PL", 6).unwrap(),
            crate::Entry::parse_line("parlare\tparlavo\tV;IND;PST;1;SG", 7).unwrap(),
            crate::Entry::parse_line("essere\tsono\tV;IND;PRS;1;SG", 8).unwrap(),
            crate::Entry::parse_line("essere\tsei\tV;IND;PRS;2;SG", 9).unwrap(),
            crate::Entry::parse_line("casa\tcasa\tN;SG", 10).unwrap(),
            crate::Entry::parse_line("casa\tcase\tN;PL", 11).unwrap(),
        ];

        store.import(&lang, &entries, None, None).unwrap();
        store
    }

    #[test]
    fn query_by_lemma() {
        let store = setup_store();
        let results = store.query("ita").lemma("parlare").execute().unwrap();
        assert_eq!(results.len(), 7);
    }

    #[test]
    fn query_by_lemma_like() {
        let store = setup_store();
        // Prefix match
        let results = store.query("ita").lemma("parl%").execute().unwrap();
        assert_eq!(results.len(), 7); // all parlare forms

        // All lemmas starting with 'e'
        let results = store.query("ita").lemma("e%").execute().unwrap();
        assert_eq!(results.len(), 2); // essere forms
    }

    #[test]
    fn query_by_form() {
        let store = setup_store();
        let results = store.query("ita").form("sono").execute().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lemma, "essere");
    }

    #[test]
    fn query_by_form_like() {
        let store = setup_store();
        // Suffix match - forms ending in "o"
        let results = store.query("ita").form("%o").execute().unwrap();
        assert_eq!(results.len(), 5); // parlo, parli(amo), parlano, parlavo, sono
    }

    #[test]
    fn query_by_lemma_and_features_contain() {
        let store = setup_store();
        let results = store
            .query("ita")
            .lemma("parlare")
            .features_contain(&["PRS", "SG"])
            .execute()
            .unwrap();
        assert_eq!(results.len(), 3); // parlo, parli, parla
    }

    #[test]
    fn query_by_features_match() {
        let store = setup_store();
        let results = store
            .query("ita")
            .features_match("V;IND;PRS;1;*")
            .execute()
            .unwrap();
        assert_eq!(results.len(), 3); // parlo, parliamo, sono
    }

    #[test]
    fn query_by_pos() {
        let store = setup_store();
        let results = store.query("ita").pos("N").execute().unwrap();
        assert_eq!(results.len(), 2); // casa, case
    }

    #[test]
    fn query_with_limit() {
        let store = setup_store();
        let results = store.query("ita").limit(3).execute().unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn query_with_offset() {
        let store = setup_store();
        let all = store.query("ita").execute().unwrap();
        let offset = store.query("ita").offset(2).execute().unwrap();
        assert_eq!(offset.len(), all.len() - 2);
    }

    #[test]
    fn query_count() {
        let store = setup_store();
        let count = store.query("ita").lemma("parlare").count().unwrap();
        assert_eq!(count, 7);
    }

    #[test]
    fn query_exists() {
        let store = setup_store();
        assert!(store.query("ita").lemma("parlare").exists().unwrap());
        assert!(!store.query("ita").lemma("nonexistent").exists().unwrap());
    }

    #[test]
    fn query_first() {
        let store = setup_store();
        let first = store.query("ita").lemma("parlare").first().unwrap();
        assert!(first.is_some());

        let none = store.query("ita").lemma("nonexistent").first().unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn query_chained_filters() {
        let store = setup_store();
        let results = store
            .query("ita")
            .pos("V")
            .features_contain(&["IND", "PRS"])
            .features_match("*;*;*;1;*")
            .execute()
            .unwrap();
        // First person indicative present verbs: parlo, parliamo, sono
        assert_eq!(results.len(), 3);
    }
}
