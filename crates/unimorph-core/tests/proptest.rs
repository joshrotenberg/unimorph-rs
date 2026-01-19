//! Property-based tests for unimorph-core.
//!
//! These tests use proptest to generate random inputs and verify
//! that invariants hold across all inputs.

use proptest::prelude::*;
use unimorph_core::{Entry, FeatureBundle, LangCode};

/// Strategy for generating valid ISO 639-3 language codes.
fn lang_code_strategy() -> impl Strategy<Value = String> {
    "[a-z]{3}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid feature strings (single feature).
fn feature_strategy() -> impl Strategy<Value = String> {
    // Features are uppercase letters, digits, and dots
    "[A-Z0-9.]{1,10}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid feature bundles.
fn feature_bundle_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(feature_strategy(), 1..=8).prop_map(|features| features.join(";"))
}

/// Strategy for generating valid lemmas/forms (non-empty Unicode strings).
fn word_strategy() -> impl Strategy<Value = String> {
    "[\\p{L}]{1,30}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid TSV lines.
fn tsv_line_strategy() -> impl Strategy<Value = String> {
    (word_strategy(), word_strategy(), feature_bundle_strategy())
        .prop_map(|(lemma, form, features)| format!("{}\t{}\t{}", lemma, form, features))
}

proptest! {
    /// Any 3-letter lowercase ASCII string is a valid language code.
    #[test]
    fn valid_lang_codes_parse(code in lang_code_strategy()) {
        let result = LangCode::new(&code);
        prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        let lang_code = result.unwrap();
        prop_assert_eq!(lang_code.as_str(), code);
    }

    /// Invalid language codes are rejected.
    #[test]
    fn invalid_lang_codes_rejected(code in "[A-Za-z0-9]{0,10}") {
        // Skip valid codes (exactly 3 lowercase)
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_lowercase()) {
            return Ok(());
        }
        let result = LangCode::new(&code);
        prop_assert!(result.is_err(), "Should have rejected: {}", code);
    }

    /// Any valid feature bundle string parses correctly.
    #[test]
    fn valid_feature_bundles_parse(bundle_str in feature_bundle_strategy()) {
        let result = FeatureBundle::new(&bundle_str);
        prop_assert!(result.is_ok(), "Failed to parse: {}", bundle_str);

        let bundle = result.unwrap();
        prop_assert_eq!(bundle.as_str(), bundle_str);
    }

    /// Parsed feature bundles round-trip correctly.
    #[test]
    fn feature_bundle_roundtrip(bundle_str in feature_bundle_strategy()) {
        let bundle = FeatureBundle::new(&bundle_str).unwrap();
        let features = bundle.features();

        // Rejoining should give original string
        let rejoined = features.join(";");
        prop_assert_eq!(rejoined, bundle_str);
    }

    /// Feature bundle contains() is consistent with features().
    #[test]
    fn feature_bundle_contains_consistent(bundle_str in feature_bundle_strategy()) {
        let bundle = FeatureBundle::new(&bundle_str).unwrap();

        for feature in bundle.features() {
            prop_assert!(
                bundle.contains(feature),
                "Bundle should contain its own feature: {}",
                feature
            );
        }
    }

    /// A bundle always matches its exact pattern.
    #[test]
    fn feature_bundle_matches_self(bundle_str in feature_bundle_strategy()) {
        let bundle = FeatureBundle::new(&bundle_str).unwrap();
        prop_assert!(
            bundle.matches_pattern(&bundle_str),
            "Bundle should match itself"
        );
    }

    /// A bundle matches an all-wildcard pattern of the same length.
    #[test]
    fn feature_bundle_matches_wildcards(bundle_str in feature_bundle_strategy()) {
        let bundle = FeatureBundle::new(&bundle_str).unwrap();
        let wildcard_pattern = vec!["*"; bundle.len()].join(";");

        prop_assert!(
            bundle.matches_pattern(&wildcard_pattern),
            "Bundle should match all-wildcard pattern"
        );
    }

    /// Valid TSV lines parse correctly.
    #[test]
    fn valid_tsv_lines_parse(line in tsv_line_strategy()) {
        let result = Entry::parse_line(&line, 1);
        prop_assert!(result.is_ok(), "Failed to parse: {}", line);

        let entry = result.unwrap();
        // Check round-trip via Display
        let displayed = format!("{}", entry);
        prop_assert_eq!(displayed, line);
    }

    /// Entry fields are preserved after parsing.
    #[test]
    fn entry_preserves_fields(
        lemma in word_strategy(),
        form in word_strategy(),
        features in feature_bundle_strategy()
    ) {
        let line = format!("{}\t{}\t{}", lemma, form, features);
        let entry = Entry::parse_line(&line, 1).unwrap();

        prop_assert_eq!(entry.lemma, lemma);
        prop_assert_eq!(entry.form, form);
        prop_assert_eq!(entry.features.as_str(), features);
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn empty_feature_bundle_rejected() {
        assert!(FeatureBundle::new("").is_err());
    }

    #[test]
    fn feature_bundle_with_empty_feature_rejected() {
        assert!(FeatureBundle::new("V;;PRS").is_err());
        assert!(FeatureBundle::new(";V;PRS").is_err());
        assert!(FeatureBundle::new("V;PRS;").is_err());
    }

    #[test]
    fn tsv_line_wrong_columns() {
        assert!(Entry::parse_line("one", 1).is_err());
        assert!(Entry::parse_line("one\ttwo", 1).is_err());
        // 4+ columns are allowed (merged into features)
        assert!(Entry::parse_line("one\ttwo\tthree\tfour", 1).is_ok());
    }

    #[test]
    fn tsv_line_empty_fields() {
        assert!(Entry::parse_line("\tform\tV;IND", 1).is_err());
        assert!(Entry::parse_line("lemma\t\tV;IND", 1).is_err());
    }

    #[test]
    fn feature_pattern_wrong_length_never_matches() {
        let bundle = FeatureBundle::new("V;IND;PRS").unwrap();
        assert!(!bundle.matches_pattern("V;IND"));
        assert!(!bundle.matches_pattern("V;IND;PRS;SG"));
    }
}
