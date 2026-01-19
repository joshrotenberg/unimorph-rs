//! Core types for UniMorph data.
//!
//! The fundamental types are:
//! - [`LangCode`]: ISO 639-3 language code (e.g., "ita", "fin")
//! - [`FeatureBundle`]: Semicolon-separated morphological features
//! - [`Entry`]: A single morphological entry (lemma, form, features)
//! - [`DatasetStats`]: Aggregate statistics for a language dataset

use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// ISO 639-3 language code (3 lowercase ASCII letters).
///
/// UniMorph uses ISO 639-3 codes to identify languages. Each code is exactly
/// 3 lowercase ASCII letters (e.g., "ita" for Italian, "fin" for Finnish).
///
/// # Examples
///
/// ```
/// use unimorph_core::LangCode;
///
/// let ita = "ita".parse::<LangCode>().unwrap();
/// assert_eq!(ita.as_str(), "ita");
///
/// // Invalid codes are rejected
/// assert!("IT".parse::<LangCode>().is_err());    // Too short, uppercase
/// assert!("italian".parse::<LangCode>().is_err()); // Too long
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
    #[inline]
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
/// Features are semicolon-separated strings like `V;IND;PRS;1;SG` representing
/// morphological properties. The UniMorph schema defines 23 dimensions of meaning
/// (tense, aspect, mood, person, number, case, gender, etc.) with 212+ features.
///
/// The bundle preserves the original string for round-tripping while also
/// parsing individual features for querying.
///
/// # Pattern Matching
///
/// The [`matches_pattern`](Self::matches_pattern) method supports wildcards:
/// - `*` matches any single feature
/// - Exact strings must match exactly
/// - Pattern length must match feature count
///
/// # Examples
///
/// ```
/// use unimorph_core::FeatureBundle;
///
/// let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
///
/// // Access individual features
/// assert_eq!(bundle.features(), &["V", "IND", "PRS", "1", "SG"]);
/// assert!(bundle.contains("IND"));
///
/// // Pattern matching with wildcards
/// assert!(bundle.matches_pattern("V;IND;*;1;*"));  // Any tense, any number
/// assert!(bundle.matches_pattern("V;*;*;*;SG"));   // Any singular verb
/// assert!(!bundle.matches_pattern("N;*;*;*;*"));   // Not a noun
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureBundle {
    raw: String,
    features: Vec<String>,
}

impl FeatureBundle {
    /// Parse a semicolon-separated feature string.
    ///
    /// Returns an error if the string is empty or contains empty features.
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

    /// Get the individual features as a slice.
    #[inline]
    pub fn features(&self) -> &[String] {
        &self.features
    }

    /// Get the number of features in the bundle.
    #[inline]
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Check if the bundle is empty (should never be true for valid bundles).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// Get the original raw string.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Check if the bundle contains a specific feature.
    ///
    /// This is an exact match - "IND" will not match "INDF".
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
    /// - `V;IND;PRS` does NOT match `V;IND;PRS;1;SG` (different lengths)
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

    /// Check if the bundle contains all of the specified features (in any order).
    ///
    /// Useful for queries like "find all indicative present verbs" without
    /// caring about the exact feature order.
    pub fn contains_all(&self, features: &[&str]) -> bool {
        features.iter().all(|f| self.contains(f))
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
/// - `lemma`: The dictionary/citation form (e.g., "parlare", "to speak")
/// - `form`: The inflected surface form (e.g., "parlo", "I speak")
/// - `features`: Morphological features (e.g., "V;IND;PRS;1;SG")
///
/// Note that the same (lemma, form) pair can appear multiple times with
/// different features. For example, a homograph might have multiple analyses.
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
    /// UniMorph format is tab-separated with at least 3 columns: lemma, form, features.
    /// Some languages have a 4th column with additional features (e.g., gender/animacy).
    /// When present, extra columns are merged into the features.
    /// No header row.
    pub fn parse_line(line: &str, line_num: usize) -> Result<Self, Error> {
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() < 3 {
            return Err(Error::MalformedEntry {
                line: line_num,
                reason: format!("expected at least 3 columns, found {}", parts.len()),
            });
        }

        let lemma = parts[0];
        let form = parts[1];
        // Merge columns 3+ into features (some languages have extra feature columns)
        // Filter out empty parts to handle trailing tabs (e.g., "lemma\tform\tfeatures\t")
        let features_str = if parts.len() > 3 {
            parts[2..]
                .iter()
                .filter(|p| !p.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(";")
        } else {
            parts[2].to_string()
        };

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

        let features = FeatureBundle::new(&features_str).map_err(|_| Error::MalformedEntry {
            line: line_num,
            reason: format!("invalid features: {}", features_str),
        })?;

        Ok(Self {
            lemma: lemma.to_string(),
            form: form.to_string(),
            features,
        })
    }

    /// Parse multiple TSV lines, returning an error on first malformed entry.
    pub fn parse_tsv(content: &str) -> Result<Vec<Self>, Error> {
        content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| Self::parse_line(line, i + 1))
            .collect()
    }

    /// Parse multiple TSV lines leniently, skipping malformed entries.
    ///
    /// Returns a tuple of (valid entries, count of skipped entries).
    /// This is useful for importing real-world data that may have errors.
    #[deprecated(
        since = "0.2.1",
        note = "use parse_tsv_with_report for detailed reporting"
    )]
    pub fn parse_tsv_lenient(content: &str) -> (Vec<Self>, usize) {
        let (entries, report) = Self::parse_tsv_with_report(content);
        (entries, report.malformed_count)
    }

    /// Parse multiple TSV lines with detailed reporting.
    ///
    /// Returns valid entries and a detailed report of what was parsed,
    /// including blank lines, malformed entries with reasons, etc.
    pub fn parse_tsv_with_report(content: &str) -> (Vec<Self>, ParseReport) {
        let mut entries = Vec::new();
        let mut report = ParseReport::new();

        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                report.blank_lines += 1;
                continue;
            }
            match Self::parse_line(line, i + 1) {
                Ok(entry) => {
                    entries.push(entry);
                    report.valid_entries += 1;
                }
                Err(Error::MalformedEntry { reason, .. }) => {
                    report.add_malformed(i + 1, reason, line);
                }
                Err(_) => {
                    report.add_malformed(i + 1, "unknown error".to_string(), line);
                }
            }
        }

        (entries, report)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}", self.lemma, self.form, self.features)
    }
}

/// Aggregate statistics for a language dataset.
///
/// These stats are pre-computed at import time and cached in the `meta` table
/// to avoid expensive full-table scans at query time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Number of unique lemmas (dictionary forms).
    pub unique_lemmas: usize,
    /// Number of unique surface forms.
    pub unique_forms: usize,
    /// Number of unique feature bundles.
    pub unique_features: usize,
}

impl DatasetStats {
    /// Create a new stats instance.
    pub fn new(
        total_entries: usize,
        unique_lemmas: usize,
        unique_forms: usize,
        unique_features: usize,
    ) -> Self {
        Self {
            total_entries,
            unique_lemmas,
            unique_forms,
            unique_features,
        }
    }

    /// Compute stats from a slice of entries.
    ///
    /// This is O(n) in the number of entries and uses hash sets internally.
    pub fn from_entries(entries: &[Entry]) -> Self {
        use std::collections::HashSet;

        let mut lemmas = HashSet::new();
        let mut forms = HashSet::new();
        let mut features = HashSet::new();

        for entry in entries {
            lemmas.insert(&entry.lemma);
            forms.insert(&entry.form);
            features.insert(entry.features.as_str());
        }

        Self {
            total_entries: entries.len(),
            unique_lemmas: lemmas.len(),
            unique_forms: forms.len(),
            unique_features: features.len(),
        }
    }
}

/// A single malformed entry with details about why it failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedEntry {
    /// Line number (1-indexed).
    pub line_num: usize,
    /// The reason the entry was rejected.
    pub reason: String,
    /// The original line content (truncated if too long).
    pub content: String,
}

impl MalformedEntry {
    /// Maximum length of content to store (to avoid memory bloat).
    const MAX_CONTENT_LEN: usize = 100;

    /// Create a new malformed entry record.
    pub fn new(line_num: usize, reason: String, content: &str) -> Self {
        let content = if content.len() > Self::MAX_CONTENT_LEN {
            format!("{}...", &content[..Self::MAX_CONTENT_LEN])
        } else {
            content.to_string()
        };
        Self {
            line_num,
            reason,
            content,
        }
    }
}

impl fmt::Display for MalformedEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}: {} ({})",
            self.line_num, self.reason, self.content
        )
    }
}

/// Compression format used for a dataset file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionFormat {
    /// No compression (raw text).
    #[default]
    None,
    /// XZ/LZMA compression (.xz).
    Xz,
    /// Gzip compression (.gz).
    Gzip,
    /// ZIP archive (.zip).
    Zip,
}

impl fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Xz => write!(f, "xz"),
            Self::Gzip => write!(f, "gzip"),
            Self::Zip => write!(f, "zip"),
        }
    }
}

/// Report from parsing TSV content.
///
/// Provides detailed breakdown of what was parsed, skipped, and why.
#[derive(Debug, Clone, Default)]
pub struct ParseReport {
    /// Number of valid entries parsed.
    pub valid_entries: usize,
    /// Number of blank lines skipped.
    pub blank_lines: usize,
    /// Number of duplicate entries (same lemma/form/features).
    pub duplicates: usize,
    /// Malformed entries with details (capped to avoid memory issues).
    pub malformed: Vec<MalformedEntry>,
    /// Total count of malformed entries (may exceed malformed.len()).
    pub malformed_count: usize,
    /// Compression format of the source file.
    pub compression: CompressionFormat,
    /// Whether the file was fetched via Git LFS.
    pub from_lfs: bool,
    /// Original filename that was downloaded.
    pub filename: Option<String>,
}

impl ParseReport {
    /// Maximum number of malformed entries to store details for.
    const MAX_MALFORMED_SAMPLES: usize = 10;

    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a malformed entry.
    pub fn add_malformed(&mut self, line_num: usize, reason: String, content: &str) {
        self.malformed_count += 1;
        if self.malformed.len() < Self::MAX_MALFORMED_SAMPLES {
            self.malformed
                .push(MalformedEntry::new(line_num, reason, content));
        }
    }

    /// Total lines processed (valid + blank + malformed).
    pub fn total_lines(&self) -> usize {
        self.valid_entries + self.blank_lines + self.malformed_count
    }

    /// Check if any issues were found.
    pub fn has_issues(&self) -> bool {
        self.malformed_count > 0
    }
}

impl fmt::Display for ParseReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Parse report:")?;
        if let Some(ref filename) = self.filename {
            write!(f, "  Source:           {}", filename)?;
            if self.compression != CompressionFormat::None {
                write!(f, " ({})", self.compression)?;
            }
            if self.from_lfs {
                write!(f, " [LFS]")?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  Valid entries:    {}", self.valid_entries)?;
        writeln!(f, "  Blank lines:      {}", self.blank_lines)?;
        if self.duplicates > 0 {
            writeln!(f, "  Duplicates:       {}", self.duplicates)?;
        }
        if self.malformed_count > 0 {
            writeln!(f, "  Malformed:        {}", self.malformed_count)?;
            for entry in &self.malformed {
                writeln!(f, "    {}", entry)?;
            }
            if self.malformed_count > self.malformed.len() {
                writeln!(
                    f,
                    "    ... and {} more",
                    self.malformed_count - self.malformed.len()
                )?;
            }
        }
        Ok(())
    }
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
            assert_eq!(bundle.len(), 5);
        }

        #[test]
        fn parse_single_feature() {
            let bundle = FeatureBundle::new("N").unwrap();
            assert_eq!(bundle.features(), &["N"]);
            assert_eq!(bundle.len(), 1);
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
        fn contains_all() {
            let bundle = FeatureBundle::new("V;IND;PRS;1;SG").unwrap();
            assert!(bundle.contains_all(&["V", "IND"]));
            assert!(bundle.contains_all(&["PRS", "SG"]));
            assert!(bundle.contains_all(&["V"]));
            assert!(!bundle.contains_all(&["V", "PL"]));
            assert!(!bundle.contains_all(&["N"]));
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
            let entry = Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap();
            assert_eq!(entry.lemma, "parlare");
            assert_eq!(entry.form, "parlo");
            assert_eq!(entry.features.as_str(), "V;IND;PRS;1;SG");
        }

        #[test]
        fn parse_unicode() {
            let entry = Entry::parse_line("essere\tè\tV;IND;PRS;3;SG", 1).unwrap();
            assert_eq!(entry.form, "è");
        }

        #[test]
        fn wrong_column_count() {
            assert!(Entry::parse_line("only_one_column", 1).is_err());
            assert!(Entry::parse_line("two\tcolumns", 1).is_err());
        }

        #[test]
        fn four_columns_merged() {
            // Some languages (e.g., Polish) have a 4th column with extra features
            let entry = Entry::parse_line("lemma\tform\tN;ACC;SG\tMASC;INAN", 1).unwrap();
            assert_eq!(entry.lemma, "lemma");
            assert_eq!(entry.form, "form");
            assert_eq!(entry.features.as_str(), "N;ACC;SG;MASC;INAN");
        }

        #[test]
        fn trailing_tab_ignored() {
            // Czech files have trailing tabs (e.g., "lemma\tform\tfeatures\t")
            let entry = Entry::parse_line("lemma\tform\tADJ;FEM;INS;DU\t", 1).unwrap();
            assert_eq!(entry.lemma, "lemma");
            assert_eq!(entry.form, "form");
            assert_eq!(entry.features.as_str(), "ADJ;FEM;INS;DU");
        }

        #[test]
        fn empty_lemma() {
            assert!(Entry::parse_line("\tform\tV;IND", 1).is_err());
        }

        #[test]
        fn empty_form() {
            assert!(Entry::parse_line("lemma\t\tV;IND", 1).is_err());
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

        #[test]
        fn parse_with_report() {
            let content =
                "parlare\tparlo\tV;IND;PRS;1;SG\nbad line\nparlare\tparli\tV;IND;PRS;2;SG\n";
            let (entries, report) = Entry::parse_tsv_with_report(content);
            assert_eq!(entries.len(), 2);
            assert_eq!(report.valid_entries, 2);
            assert_eq!(report.malformed_count, 1);
            assert_eq!(report.malformed.len(), 1);
            assert_eq!(report.malformed[0].line_num, 2);
            assert!(
                report.malformed[0]
                    .reason
                    .contains("expected at least 3 columns")
            );
        }

        #[test]
        fn display() {
            let entry = Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap();
            assert_eq!(format!("{}", entry), "parlare\tparlo\tV;IND;PRS;1;SG");
        }
    }

    mod dataset_stats {
        use super::*;

        #[test]
        fn from_entries() {
            let entries = vec![
                Entry::parse_line("parlare\tparlo\tV;IND;PRS;1;SG", 1).unwrap(),
                Entry::parse_line("parlare\tparli\tV;IND;PRS;2;SG", 2).unwrap(),
                Entry::parse_line("essere\tsono\tV;IND;PRS;1;SG", 3).unwrap(),
            ];

            let stats = DatasetStats::from_entries(&entries);
            assert_eq!(stats.total_entries, 3);
            assert_eq!(stats.unique_lemmas, 2); // parlare, essere
            assert_eq!(stats.unique_forms, 3); // parlo, parli, sono
            assert_eq!(stats.unique_features, 2); // V;IND;PRS;1;SG, V;IND;PRS;2;SG
        }
    }
}
