//! CLI integration tests.
//!
//! These tests verify the CLI binary behaves correctly.

use assert_cmd::Command;
use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

fn unimorph() -> Command {
    cargo_bin_cmd!("unimorph")
}

mod help {
    use super::*;

    #[test]
    fn help_shows_usage() {
        unimorph()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"))
            .stdout(predicate::str::contains("Commands:"));
    }

    #[test]
    fn help_shows_all_commands() {
        unimorph()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("download"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("inflect"))
            .stdout(predicate::str::contains("analyze"))
            .stdout(predicate::str::contains("stats"))
            .stdout(predicate::str::contains("delete"));
    }

    #[test]
    fn version_shows_version() {
        unimorph()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("unimorph"))
            .stdout(predicate::str::contains("0.1.0"));
    }

    #[test]
    fn download_help() {
        unimorph()
            .args(["download", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[LANG]"))
            .stdout(predicate::str::contains("--force"));
    }

    #[test]
    fn inflect_help() {
        unimorph()
            .args(["inflect", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--lang"))
            .stdout(predicate::str::contains("<LEMMA>"))
            .stdout(predicate::str::contains("--features"))
            .stdout(predicate::str::contains("--json"));
    }

    #[test]
    fn analyze_help() {
        unimorph()
            .args(["analyze", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--lang"))
            .stdout(predicate::str::contains("<FORM>"))
            .stdout(predicate::str::contains("--json"));
    }

    #[test]
    fn stats_help() {
        unimorph()
            .args(["stats", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[LANG]"))
            .stdout(predicate::str::contains("--json"));
    }

    #[test]
    fn list_help() {
        unimorph()
            .args(["list", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--cached"));
    }

    #[test]
    fn delete_help() {
        unimorph()
            .args(["delete", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[LANG]"));
    }
}

mod errors {
    use super::*;

    #[test]
    fn missing_command_shows_help() {
        unimorph()
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage:"));
    }

    #[test]
    fn unknown_command_shows_error() {
        unimorph()
            .arg("unknown")
            .assert()
            .failure()
            .stderr(predicate::str::contains("error"));
    }

    #[test]
    fn download_without_lang_shows_helpful_error() {
        // Without a default language set, should show helpful error
        unimorph()
            .arg("download")
            .assert()
            .failure()
            .stderr(predicate::str::contains("No language specified"));
    }

    #[test]
    fn inflect_requires_lemma() {
        unimorph()
            .arg("inflect")
            .assert()
            .failure()
            .stderr(predicate::str::contains("<LEMMA>"));
    }

    #[test]
    fn inflect_with_lemma_but_no_lang_shows_helpful_error() {
        unimorph()
            .args(["inflect", "parlare"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("No language specified"));
    }

    #[test]
    fn analyze_requires_form() {
        unimorph()
            .arg("analyze")
            .assert()
            .failure()
            .stderr(predicate::str::contains("<FORM>"));
    }

    #[test]
    fn analyze_with_form_but_no_lang_shows_helpful_error() {
        unimorph()
            .args(["analyze", "parlo"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("No language specified"));
    }

    #[test]
    fn stats_without_lang_shows_helpful_error() {
        unimorph()
            .arg("stats")
            .assert()
            .failure()
            .stderr(predicate::str::contains("No language specified"));
    }

    #[test]
    fn delete_without_lang_shows_helpful_error() {
        unimorph()
            .arg("delete")
            .assert()
            .failure()
            .stderr(predicate::str::contains("No language specified"));
    }
}

mod list_command {
    use super::*;

    #[test]
    fn list_shows_cached_by_default() {
        // Default behavior shows cached languages (pipe-friendly when not TTY)
        // or nothing if no languages are cached
        unimorph().arg("list").assert().success();
    }

    #[test]
    fn list_cached_works() {
        // This test uses the default cache, so results depend on what's downloaded
        unimorph().args(["list", "--cached"]).assert().success();
    }
}

mod config_command {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn config_help() {
        unimorph()
            .args(["config", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("show"))
            .stdout(predicate::str::contains("init"))
            .stdout(predicate::str::contains("path"));
    }

    #[test]
    fn config_show_works() {
        unimorph()
            .args(["config", "show"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration"));
    }

    #[test]
    fn config_show_json() {
        unimorph()
            .args(["config", "show", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"path\""))
            .stdout(predicate::str::contains("\"config\""));
    }

    #[test]
    fn config_path_works() {
        unimorph()
            .args(["config", "path"])
            .assert()
            .success()
            .stdout(predicate::str::contains("config.toml"));
    }

    #[test]
    fn config_path_json() {
        unimorph()
            .args(["config", "path", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"path\""))
            .stdout(predicate::str::contains("\"exists\""));
    }

    /// Get the config directory path (always ~/.config/unimorph on all platforms).
    fn config_dir_for_temp(temp_dir: &TempDir) -> std::path::PathBuf {
        temp_dir.path().join(".config").join("unimorph")
    }

    #[test]
    fn config_init_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = config_dir_for_temp(&temp_dir);

        unimorph()
            .env("HOME", temp_dir.path())
            .args(["config", "init"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Success"));

        assert!(config_dir.join("config.toml").exists());
    }

    #[test]
    fn config_init_refuses_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = config_dir_for_temp(&temp_dir);
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "# existing").unwrap();

        unimorph()
            .env("HOME", temp_dir.path())
            .args(["config", "init"])
            .assert()
            .success()
            .stdout(predicate::str::contains("already exists"));
    }

    #[test]
    fn config_init_force_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = config_dir_for_temp(&temp_dir);
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "# existing").unwrap();

        unimorph()
            .env("HOME", temp_dir.path())
            .args(["config", "init", "--force"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Success"));

        let content = std::fs::read_to_string(config_dir.join("config.toml")).unwrap();
        assert!(content.contains("UniMorph CLI Configuration"));
    }

    #[test]
    fn config_alias_works() {
        unimorph()
            .args(["cfg", "show"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration"));
    }
}

// Tests that require network access or modify state should be marked #[ignore]
// and run separately with `cargo test -- --ignored`
mod network_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[ignore = "requires network access"]
    fn download_and_query_workflow() {
        let temp_dir = TempDir::new().unwrap();

        // Download a small dataset (Czech is relatively small)
        unimorph()
            .env("HOME", temp_dir.path())
            .args(["download", "ces"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Downloaded ces"));

        // Query should work now
        unimorph()
            .env("HOME", temp_dir.path())
            .args(["stats", "ces"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Total entries"));
    }
}
