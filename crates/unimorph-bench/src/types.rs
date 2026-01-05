//! Core types for UniMorph data.
//!
//! These types will migrate to `unimorph-core` once we've validated
//! the design through benchmarking.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// ISO 639-3 language code (3 lowercase ASCII letters).
///
/// # Examples
///
/// ```
/// use unimorph_bench::LangCode;
///
/// let ita = "ita".parse::<LangCode>().unwrap();
/// assert_eq!(ita.as_str(), "ita");
///
/// // Invalid codes are rejected
/// assert!("IT".parse::<LangCode>().is_err());
/// assert!("italian".parse::<LangCode>().is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LangCode(String);

impl LangCode {
    /// Create a new language code, validating the format.
    ///
    /// Returns an error if the code is not exactly 3 lowercase ASCII letters.
    pub fn new(s: &str) -> Result<Self, Error> {
        if s.len() != 3 {
            return Err(Error::InvalidLangCode(s.to_string()));
        }
        if !s.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(Error::InvalidLangCode(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }

    /// Get the language code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for LangCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for LangCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for LangCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A bundle of morphological features.
///
/// Features are semicolon-separated strings like `V;IND;PRS;1;SG`.
/// The bundle preserves the original string for round-tripping while
/// also parsing individual features for querying.
///
/// # Pattern Matching
///
/// The `matches_pattern` method supports wildcards:
/// - `*` matches any single feature
/// - Exact strings must match exactly
///
/// # Examples
///
/// ```
/// use unimorph_bench::FeatureBundle;
///
/// let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
/// assert_eq!(bundle.features(), &["V", "IND", "PRS", "1", "SG"]);
/// assert!(bundle.contains("IND"));
/// assert!(bundle.matches_pattern("V;IND;*;1;*"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureBundle {
    raw: String,
    features: Vec<String>,
}

impl FeatureBundle {
    /// Parse a semicolon-separated feature string.
    pub fn new(s: &str) -> Result<Self, Error> {
        if s.is_empty() {
            return Err(Error::InvalidFeatureBundle(
                "empty feature bundle".to_string(),
            ));
        }

        let features: Vec<String> = s.split(';').map(|f| f.to_string()).collect();

        // Validate: no empty features
        if features.iter().any(|f| f.is_empty()) {
            return Err(Error::InvalidFeatureBundle(format!(
                "empty feature in bundle: {}",
                s
            )));
        }

        Ok(Self {
            raw: s.to_string(),
            features,
        })
    }

    /// Get the individual features.
    pub fn features(&self) -> &[String] {
        &self.features
    }

    /// Get the original raw string.
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Check if the bundle contains a specific feature.
    pub fn contains(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    /// Check if the bundle matches a pattern with wildcards.
    ///
    /// The pattern uses `;` as separator and `*` as wildcard.
    /// The number of pattern elements must match the number of features.
    ///
    /// # Examples
    ///
    /// - `V;IND;*;1;*` matches `V;IND;PRS;1;SG` and `V;IND;PST;1;PL`
    /// - `N;*` matches any noun with exactly 2 features
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split(';').collect();

        if pattern_parts.len() != self.features.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(self.features.iter())
            .all(|(pat, feat)| *pat == "*" || *pat == feat)
    }
}

impl fmt::Display for FeatureBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl AsRef<str> for FeatureBundle {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

/// A single morphological entry from UniMorph.
///
/// Each entry is a triple of (lemma, form, features):
/// - `lemma`: The dictionary/citation form (e.g., "parlare")
/// - `form`: The inflected surface form (e.g., "parlo")
/// - `features`: Morphological features (e.g., "V;IND;PRS;1;SG")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The dictionary/citation form.
    pub lemma: String,
    /// The inflected surface form.
    pub form: String,
    /// The morphological features.
    pub features: FeatureBundle,
}

impl Entry {
    /// Create a new entry.
    pub fn new(lemma: String, form: String, features: FeatureBundle) -> Self {
        Self {
            lemma,
            form,
            features,
        }
    }

    /// Parse a TSV line into an entry.
    ///
    /// UniMorph format is tab-separated with 3 columns: lemma, form, features.
    /// No header row.
    pub fn parse_tsv_line(line: &str, line_num: usize) -> Result<Self, Error> {
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() != 3 {
            return Err(Error::MalformedEntry {
                line: line_num,
                reason: format!("expected 3 columns, found {}", parts.len()),
            });
        }

        let lemma = parts[0];
        let form = parts[1];
        let features_str = parts[2];

        if lemma.is_empty() {
            return Err(Error::MalformedEntry {
                line: line_num,
                reason: "empty lemma".to_string(),
            });
        }

        if form.is_empty() {
            return Err(Error::MalformedEntry {
                line: line_num,
                reason: "empty form".to_string(),
            });
        }

        let features = FeatureBundle::new(features_str).map_err(|_| Error::MalformedEntry {
            line: line_num,
            reason: format!("invalid features: {}", features_str),
        })?;

        Ok(Self {
            lemma: lemma.to_string(),
            form: form.to_string(),
            features,
        })
    }

    /// Parse multiple TSV lines, skipping empty lines.
    pub fn parse_tsv(content: &str) -> Result<Vec<Self>, Error> {
        content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| Self::parse_tsv_line(line, i + 1))
            .collect()
    }

    /// Parse multiple TSV lines leniently, skipping malformed entries.
    ///
    /// Returns a tuple of (valid entries, count of skipped entries).
    /// This is useful for benchmarking with real-world data that may have errors.
    pub fn parse_tsv_lenient(content: &str) -> (Vec<Self>, usize) {
        let mut entries = Vec::new();
        let mut skipped = 0;

        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match Self::parse_tsv_line(line, i + 1) {
                Ok(entry) => entries.push(entry),
                Err(_) => skipped += 1,
            }
        }

        (entries, skipped)
    }
}

/// Aggregate statistics for a dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Number of unique lemmas.
    pub unique_lemmas: usize,
    /// Number of unique surface forms.
    pub unique_forms: usize,
    /// Number of unique feature bundles.
    pub unique_features: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod lang_code {
        use super::*;

        #[test]
        fn valid_codes() {
            assert!(LangCode::new("ita").is_ok());
            assert!(LangCode::new("eng").is_ok());
            assert!(LangCode::new("deu").is_ok());
            assert!(LangCode::new("fin").is_ok());
        }

        #[test]
        fn invalid_length() {
            assert!(LangCode::new("it").is_err());
            assert!(LangCode::new("italian").is_err());
            assert!(LangCode::new("").is_err());
        }

        #[test]
        fn invalid_characters() {
            assert!(LangCode::new("ITA").is_err());
            assert!(LangCode::new("It1").is_err());
            assert!(LangCode::new("it-").is_err());
        }

        #[test]
        fn display() {
            let code = LangCode::new("ita").unwrap();
            assert_eq!(format!("{}", code), "ita");
        }

        #[test]
        fn from_str() {
            let code: LangCode = "ita".parse().unwrap();
            assert_eq!(code.as_str(), "ita");
        }
    }

    mod feature_bundle {
        use super::*;

        #[test]
        fn parse_simple() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert_eq!(bundle.features(), &["V", "IND", "PRS", "1", "SG"]);
            assert_eq!(bundle.as_str(), "V;IND;PRS;1;SG");
        }

        #[test]
        fn parse_single_feature() {
            let bundle = FeatureBundle::new("N").unwrap();
            assert_eq!(bundle.features(), &["N"]);
        }

        #[test]
        fn empty_bundle_rejected() {
            assert!(FeatureBundle::new("").is_err());
        }

        #[test]
        fn empty_feature_rejected() {
            assert!(FeatureBundle::new("V;;PRS").is_err());
            assert!(FeatureBundle::new(";V;PRS").is_err());
            assert!(FeatureBundle::new("V;PRS;").is_err());
        }

        #[test]
        fn contains() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert!(bundle.contains("V"));
            assert!(bundle.contains("IND"));
            assert!(bundle.contains("SG"));
            assert!(!bundle.contains("PL"));
            assert!(!bundle.contains("SBJV"));
        }

        #[test]
        fn matches_pattern_exact() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert!(bundle.matches_pattern("V;IND;PRS;1;SG"));
            assert!(!bundle.matches_pattern("V;IND;PRS;1;PL"));
        }

        #[test]
        fn matches_pattern_wildcards() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert!(bundle.matches_pattern("V;*;*;*;*"));
            assert!(bundle.matches_pattern("*;IND;*;1;*"));
            assert!(bundle.matches_pattern("V;IND;*;1;SG"));
            assert!(bundle.matches_pattern("*;*;*;*;*"));
        }

        #[test]
        fn matches_pattern_wrong_length() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert!(!bundle.matches_pattern("V;IND;PRS"));
            assert!(!bundle.matches_pattern("V;IND;PRS;1;SG;EXTRA"));
        }
    }

    mod entry {
        use super::*;

        #[test]
        fn parse_valid_line() {
            let entry = Entry::parse_tsv_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap();
            assert_eq!(entry.lemma, "parlare");
            assert_eq!(entry.form, "parlo");
            assert_eq!(entry.features.as_str(), "V;IND;PRS;1;SG");
        }

        #[test]
        fn parse_unicode() {
            let entry = Entry::parse_tsv_line("essere\tè\tV;IND;PRS;3;SG", 1).unwrap();
            assert_eq!(entry.form, "è");
        }

        #[test]
        fn wrong_column_count() {
            assert!(Entry::parse_tsv_line("only_one_column", 1).is_err());
            assert!(Entry::parse_tsv_line("two\tcolumns", 1).is_err());
            assert!(Entry::parse_tsv_line("a\tb\tc\td", 1).is_err());
        }

        #[test]
        fn empty_lemma() {
            assert!(Entry::parse_tsv_line("\tform\tV;IND", 1).is_err());
        }

        #[test]
        fn empty_form() {
            assert!(Entry::parse_tsv_line("lemma\t\tV;IND", 1).is_err());
        }

        #[test]
        fn parse_multiple_lines() {
            let content = "parlare\tparlo\tV;IND;PRS;1;SG\nparlare\tparli\tV;IND;PRS;2;SG\n";
            let entries = Entry::parse_tsv(content).unwrap();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].form, "parlo");
            assert_eq!(entries[1].form, "parli");
        }

        #[test]
        fn parse_skips_empty_lines() {
            let content = "parlare\tparlo\tV;IND;PRS;1;SG\n\nparlare\tparli\tV;IND;PRS;2;SG\n";
            let entries = Entry::parse_tsv(content).unwrap();
            assert_eq!(entries.len(), 2);
        }
    }
}
