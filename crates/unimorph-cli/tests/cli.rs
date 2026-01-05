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
            .stdout(predicate::str::contains("--lang"))
            .stdout(predicate::str::contains("--force"));
    }

    #[test]
    fn inflect_help() {
        unimorph()
            .args(["inflect", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--lang"))
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
            .stdout(predicate::str::contains("--json"));
    }

    #[test]
    fn stats_help() {
        unimorph()
            .args(["stats", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--lang"))
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
            .stdout(predicate::str::contains("--lang"));
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
    fn download_requires_lang() {
        unimorph()
            .arg("download")
            .assert()
            .failure()
            .stderr(predicate::str::contains("--lang"));
    }

    #[test]
    fn inflect_requires_lang() {
        unimorph()
            .args(["inflect", "parlare"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("--lang"));
    }

    #[test]
    fn inflect_requires_lemma() {
        unimorph().args(["inflect", "-l", "ita"]).assert().failure();
    }

    #[test]
    fn analyze_requires_lang() {
        unimorph()
            .args(["analyze", "parlo"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("--lang"));
    }

    #[test]
    fn analyze_requires_form() {
        unimorph().args(["analyze", "-l", "ita"]).assert().failure();
    }

    #[test]
    fn stats_requires_lang() {
        unimorph()
            .arg("stats")
            .assert()
            .failure()
            .stderr(predicate::str::contains("--lang"));
    }

    #[test]
    fn delete_requires_lang() {
        unimorph()
            .arg("delete")
            .assert()
            .failure()
            .stderr(predicate::str::contains("--lang"));
    }
}

mod list_command {
    use super::*;

    #[test]
    fn list_shows_available_info() {
        unimorph()
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("github.com/unimorph"));
    }

    #[test]
    fn list_cached_works() {
        // This test uses the default cache, so results depend on what's downloaded
        unimorph().args(["list", "--cached"]).assert().success();
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
            .args(["download", "-l", "ces"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Downloaded ces"));

        // Query should work now
        unimorph()
            .env("HOME", temp_dir.path())
            .args(["stats", "-l", "ces"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Total entries"));
    }
}
