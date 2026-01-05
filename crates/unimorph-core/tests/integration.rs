//! Integration tests for unimorph-core.
//!
//! These tests verify complete workflows and interactions between components.

use tempfile::TempDir;
use unimorph_core::{Entry, LangCode, Repository, Store};

/// Sample Italian verb data for testing.
const SAMPLE_ITA_DATA: &str = r#"parlare	parlo	V;IND;PRS;1;SG
parlare	parli	V;IND;PRS;2;SG
parlare	parla	V;IND;PRS;3;SG
parlare	parliamo	V;IND;PRS;1;PL
parlare	parlate	V;IND;PRS;2;PL
parlare	parlano	V;IND;PRS;3;PL
essere	sono	V;IND;PRS;1;SG
essere	sei	V;IND;PRS;2;SG
essere	è	V;IND;PRS;3;SG
essere	siamo	V;IND;PRS;1;PL
essere	siete	V;IND;PRS;2;PL
essere	sono	V;IND;PRS;3;PL
"#;

/// Sample German noun data for testing.
const SAMPLE_DEU_DATA: &str = r#"Haus	Haus	N;NOM;SG
Haus	Hauses	N;GEN;SG
Haus	Haus	N;DAT;SG
Haus	Haus	N;ACC;SG
Haus	Häuser	N;NOM;PL
Haus	Häuser	N;GEN;PL
Haus	Häusern	N;DAT;PL
Haus	Häuser	N;ACC;PL
"#;

fn setup_store_with_data(data: &str, lang: &str) -> (TempDir, Store) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut store = Store::open(&db_path).unwrap();

    let entries = Entry::parse_tsv(data).unwrap();
    let lang_code = LangCode::new(lang).unwrap();
    store.import(&lang_code, &entries, None).unwrap();

    (temp_dir, store)
}

mod store_integration {
    use super::*;

    #[test]
    fn inflect_returns_all_forms() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        let forms = store.inflect("ita", "parlare").unwrap();
        assert_eq!(forms.len(), 6);

        let form_strings: Vec<_> = forms.iter().map(|e| e.form.as_str()).collect();
        assert!(form_strings.contains(&"parlo"));
        assert!(form_strings.contains(&"parli"));
        assert!(form_strings.contains(&"parla"));
        assert!(form_strings.contains(&"parliamo"));
        assert!(form_strings.contains(&"parlate"));
        assert!(form_strings.contains(&"parlano"));
    }

    #[test]
    fn analyze_returns_correct_lemma() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        let analyses = store.analyze("ita", "parlo").unwrap();
        assert_eq!(analyses.len(), 1);
        assert_eq!(analyses[0].lemma, "parlare");
        assert_eq!(analyses[0].features.as_str(), "V;IND;PRS;1;SG");
    }

    #[test]
    fn analyze_handles_ambiguous_forms() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        // "sono" appears twice: 1SG and 3PL of "essere"
        let analyses = store.analyze("ita", "sono").unwrap();
        assert_eq!(analyses.len(), 2);
        assert!(analyses.iter().all(|e| e.lemma == "essere"));
    }

    #[test]
    fn stats_are_accurate() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        let stats = store.stats("ita").unwrap().unwrap();
        assert_eq!(stats.total_entries, 12);
        assert_eq!(stats.unique_lemmas, 2); // parlare, essere
        assert_eq!(stats.unique_forms, 11); // "sono" appears twice
    }

    #[test]
    fn multiple_languages_isolated() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut store = Store::open(&db_path).unwrap();

        // Import Italian
        let ita_entries = Entry::parse_tsv(SAMPLE_ITA_DATA).unwrap();
        let ita = LangCode::new("ita").unwrap();
        store.import(&ita, &ita_entries, None).unwrap();

        // Import German
        let deu_entries = Entry::parse_tsv(SAMPLE_DEU_DATA).unwrap();
        let deu = LangCode::new("deu").unwrap();
        store.import(&deu, &deu_entries, None).unwrap();

        // Italian queries don't return German data
        let ita_forms = store.inflect("ita", "Haus").unwrap();
        assert!(ita_forms.is_empty());

        // German queries don't return Italian data
        let deu_forms = store.inflect("deu", "parlare").unwrap();
        assert!(deu_forms.is_empty());

        // Each language has correct stats
        let ita_stats = store.stats("ita").unwrap().unwrap();
        let deu_stats = store.stats("deu").unwrap().unwrap();
        assert_eq!(ita_stats.total_entries, 12);
        assert_eq!(deu_stats.total_entries, 8);

        // Languages list is correct
        let langs = store.languages().unwrap();
        assert_eq!(langs.len(), 2);
    }

    #[test]
    fn search_features_exact_match() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        let results = store.search_features("ita", "V;IND;PRS;1;SG").unwrap();
        assert_eq!(results.len(), 2); // parlo, sono

        let forms: Vec<_> = results.iter().map(|e| e.form.as_str()).collect();
        assert!(forms.contains(&"parlo"));
        assert!(forms.contains(&"sono"));
    }

    #[test]
    fn search_features_with_wildcard() {
        let (_dir, store) = setup_store_with_data(SAMPLE_ITA_DATA, "ita");

        // All singular forms
        let results = store.search_features("ita", "V;IND;PRS;*;SG").unwrap();
        assert_eq!(results.len(), 6); // 3 per verb * 2 verbs
    }

    #[test]
    fn delete_language_removes_all_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut store = Store::open(&db_path).unwrap();

        let entries = Entry::parse_tsv(SAMPLE_ITA_DATA).unwrap();
        let lang = LangCode::new("ita").unwrap();
        store.import(&lang, &entries, None).unwrap();

        assert!(store.has_language("ita").unwrap());
        assert_eq!(store.stats("ita").unwrap().unwrap().total_entries, 12);

        store.delete_language("ita").unwrap();

        assert!(!store.has_language("ita").unwrap());
        assert!(store.stats("ita").unwrap().is_none());
        assert!(store.inflect("ita", "parlare").unwrap().is_empty());
    }

    #[test]
    fn reimport_replaces_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut store = Store::open(&db_path).unwrap();

        let lang = LangCode::new("ita").unwrap();

        // First import
        let entries1 = Entry::parse_tsv(SAMPLE_ITA_DATA).unwrap();
        store.import(&lang, &entries1, None).unwrap();
        assert_eq!(store.stats("ita").unwrap().unwrap().total_entries, 12);

        // Second import with different data
        let entries2 = Entry::parse_tsv("nuovo\tnuova\tADJ;FEM;SG\n").unwrap();
        store.import(&lang, &entries2, None).unwrap();
        assert_eq!(store.stats("ita").unwrap().unwrap().total_entries, 1);

        // Old data is gone
        assert!(store.inflect("ita", "parlare").unwrap().is_empty());
    }
}

mod repository_integration {
    use super::*;

    #[test]
    fn repository_creates_cache_dir() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("unimorph_cache");

        assert!(!cache_path.exists());
        let _repo = Repository::with_cache_dir(&cache_path).unwrap();
        assert!(cache_path.exists());
        assert!(cache_path.join("datasets.db").exists());
    }

    #[test]
    fn repository_lists_empty_cache() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::with_cache_dir(temp_dir.path()).unwrap();

        let langs = repo.cached_languages().unwrap();
        assert!(langs.is_empty());
    }
}

mod parsing_edge_cases {
    use super::*;

    #[test]
    fn parse_unicode_forms() {
        let data = "être\tsuis\tV;IND;PRS;1;SG\n";
        let entries = Entry::parse_tsv(data).unwrap();
        assert_eq!(entries[0].lemma, "être");
        assert_eq!(entries[0].form, "suis");
    }

    #[test]
    fn parse_unicode_with_diacritics() {
        let data = "ação\tações\tN;PL\n";
        let entries = Entry::parse_tsv(data).unwrap();
        assert_eq!(entries[0].lemma, "ação");
        assert_eq!(entries[0].form, "ações");
    }

    #[test]
    fn parse_cyrillic() {
        let data = "говорить\tговорю\tV;IND;PRS;1;SG\n";
        let entries = Entry::parse_tsv(data).unwrap();
        assert_eq!(entries[0].lemma, "говорить");
        assert_eq!(entries[0].form, "говорю");
    }

    #[test]
    fn parse_arabic() {
        let data = "كتب\tيكتب\tV;IND;PRS;3;SG;MASC\n";
        let entries = Entry::parse_tsv(data).unwrap();
        assert_eq!(entries[0].lemma, "كتب");
        assert_eq!(entries[0].form, "يكتب");
    }

    #[test]
    fn parse_lenient_skips_bad_lines() {
        let data = "good\tentry\tV;IND\nbad line\nanother\tgood\tN;SG\n";
        let (entries, skipped) = Entry::parse_tsv_lenient(data);
        assert_eq!(entries.len(), 2);
        assert_eq!(skipped, 1);
    }

    #[test]
    fn parse_lenient_handles_empty_lines() {
        let data = "good\tentry\tV;IND\n\n\nanother\tgood\tN;SG\n";
        let (entries, skipped) = Entry::parse_tsv_lenient(data);
        assert_eq!(entries.len(), 2);
        assert_eq!(skipped, 0);
    }
}
