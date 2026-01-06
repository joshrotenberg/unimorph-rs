//! Repair command implementation.

use std::path::Path;

use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::util::create_repo;

/// Get the path to the available languages cache file.
fn available_languages_cache_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|p| p.join("unimorph").join("available_languages.json"))
}

#[instrument(skip_all)]
pub fn cmd_repair(
    clear_cache: bool,
    clear_data: bool,
    json: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    let mut actions_taken = Vec::new();

    // Clear cache files
    if clear_cache && let Some(cache_path) = available_languages_cache_path() {
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)?;
            info!(path = %cache_path.display(), "removed cache file");
            actions_taken.push(format!("Removed cache: {}", cache_path.display()));
        } else {
            actions_taken.push("No cache file found".to_string());
        }
    }

    // Clear data (SQLite database)
    if clear_data {
        let repo = create_repo(data_dir)?;
        let db_path = repo.cache_dir().join("datasets.db");

        // Get list of languages before deleting
        let languages: Vec<String> = repo
            .cached_languages()
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.to_string())
            .collect();

        // Close the connection by dropping repo
        drop(repo);

        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
            info!(path = %db_path.display(), "removed database file");

            if languages.is_empty() {
                actions_taken.push(format!("Removed database: {}", db_path.display()));
            } else {
                actions_taken.push(format!(
                    "Removed database ({} language(s): {})",
                    languages.len(),
                    languages.join(", ")
                ));
            }
        } else {
            actions_taken.push("No database found".to_string());
        }
    }

    // Run integrity check if not clearing data
    if !clear_data && !clear_cache {
        let repo = create_repo(data_dir)?;
        let db_path = repo.cache_dir().join("datasets.db");

        if db_path.exists() {
            // Check database is accessible by querying cached languages
            match repo.cached_languages() {
                Ok(langs) => {
                    actions_taken.push(format!("Database OK ({} language(s) cached)", langs.len()));
                }
                Err(e) => {
                    actions_taken.push(format!("Database error: {}", e));
                    println!();
                    println!("The database may be corrupted. Run with --clear-data to rebuild:");
                    println!("  unimorph repair --clear-data");
                }
            }
        } else {
            actions_taken.push("No database found (nothing to repair)".to_string());
        }
    }

    // Print summary
    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": if actions_taken.is_empty() { "no_action" } else { "complete" },
                "actions": actions_taken
            })
        );
    } else if actions_taken.is_empty() {
        println!("Nothing to do. Use --clear-cache or --clear-data to clear data.");
        println!();
        println!("Options:");
        println!("  --clear-cache  Remove cached API responses");
        println!("  --clear-data   Remove downloaded datasets (will need to re-download)");
    } else {
        println!("Repair complete:");
        for action in &actions_taken {
            println!("  {}", action);
        }

        if clear_data {
            println!();
            println!("To re-download languages:");
            println!("  unimorph download <lang>");
        }
    }

    Ok(())
}
