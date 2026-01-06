//! Features command implementation.

use std::collections::HashMap;
use std::path::Path;

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::util::{create_repo, require_language, validate_lang_code};

#[instrument(skip_all, fields(lang))]
#[allow(clippy::too_many_arguments)]
pub fn cmd_features(
    lang: &str,
    list: bool,
    stats: bool,
    search: Option<&str>,
    position: Option<usize>,
    limit: usize,
    json: bool,
    data_dir: Option<&Path>,
) -> Result<()> {
    validate_lang_code(lang)?;

    let repo = create_repo(data_dir)?;
    require_language(&repo, lang)?;

    // Collect all feature bundles
    let entries = repo
        .store()
        .query(lang)
        .limit(1_000_000) // Get all entries
        .execute()?;

    if list {
        // List unique feature values
        let mut all_features: Vec<String> = entries
            .iter()
            .flat_map(|e| e.features.as_str().split(';'))
            .map(|s| s.to_string())
            .collect();
        all_features.sort();
        all_features.dedup();

        if json {
            println!("{}", serde_json::to_string_pretty(&all_features)?);
        } else {
            println!("Unique features in {}:", lang);
            println!();
            for feat in &all_features {
                println!("  {}", feat);
            }
            println!();
            println!("{} unique feature values.", all_features.len());
        }
        return Ok(());
    }

    if stats {
        // Show feature value counts
        let mut counts: HashMap<String, usize> = HashMap::new();
        for entry in &entries {
            for feat in entry.features.as_str().split(';') {
                *counts.entry(feat.to_string()).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        if json {
            let map: HashMap<_, _> = sorted.into_iter().collect();
            println!("{}", serde_json::to_string_pretty(&map)?);
        } else {
            println!("Feature statistics for {}:", lang);
            println!();
            println!("{:<20} COUNT", "FEATURE");
            println!("{}", "-".repeat(40));
            for (feat, count) in sorted.iter().take(limit) {
                println!("{:<20} {}", feat, count);
            }
            if sorted.len() > limit {
                println!("... and {} more", sorted.len() - limit);
            }
        }
        return Ok(());
    }

    if let Some(term) = search {
        // Search for entries containing a specific feature
        let matching: Vec<_> = entries
            .iter()
            .filter(|e| e.features.as_str().split(';').any(|f| f == term))
            .take(limit)
            .cloned()
            .collect();

        let total_count = entries
            .iter()
            .filter(|e| e.features.as_str().split(';').any(|f| f == term))
            .count();

        if json {
            println!("{}", serde_json::to_string_pretty(&matching)?);
        } else {
            if matching.is_empty() {
                println!("No entries with feature '{}' found.", term);
                return Ok(());
            }
            println!("Entries with feature '{}':", term);
            println!();
            println!("{:<20} {:<20} FEATURES", "LEMMA", "FORM");
            println!("{}", "-".repeat(60));
            for entry in &matching {
                println!("{:<20} {:<20} {}", entry.lemma, entry.form, entry.features);
            }
            println!();
            if total_count > limit {
                println!("Showing {} of {} results.", limit, total_count);
            } else {
                println!("{} result(s).", matching.len());
            }
        }
        return Ok(());
    }

    if let Some(pos) = position {
        // Show feature values at a specific position
        let mut values: HashMap<String, usize> = HashMap::new();
        for entry in &entries {
            if let Some(feat) = entry.features.as_str().split(';').nth(pos) {
                *values.entry(feat.to_string()).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = values.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        if json {
            let map: HashMap<_, _> = sorted.into_iter().collect();
            println!("{}", serde_json::to_string_pretty(&map)?);
        } else {
            println!("Feature values at position {} in {}:", pos, lang);
            println!();
            println!("{:<20} COUNT", "VALUE");
            println!("{}", "-".repeat(40));
            for (val, count) in sorted.iter().take(limit) {
                println!("{:<20} {}", val, count);
            }
            if sorted.len() > limit {
                println!("... and {} more", sorted.len() - limit);
            }
        }
        return Ok(());
    }

    // Default: show a summary of feature structure
    let mut position_counts: Vec<HashMap<String, usize>> = Vec::new();
    for entry in &entries {
        for (i, feat) in entry.features.as_str().split(';').enumerate() {
            while position_counts.len() <= i {
                position_counts.push(HashMap::new());
            }
            *position_counts[i].entry(feat.to_string()).or_insert(0) += 1;
        }
    }

    if json {
        let summary: Vec<_> = position_counts
            .iter()
            .enumerate()
            .map(|(i, counts)| {
                serde_json::json!({
                    "position": i,
                    "unique_values": counts.len(),
                    "top_values": counts.iter()
                        .map(|(k, v)| (k.clone(), *v))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .take(5)
                        .collect::<HashMap<_, _>>()
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("Feature structure for {}:", lang);
        println!();
        for (i, counts) in position_counts.iter().enumerate() {
            let mut sorted: Vec<_> = counts.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            let top: Vec<_> = sorted.iter().take(3).map(|(k, _)| k.as_str()).collect();
            println!(
                "  Position {}: {} unique values (e.g., {})",
                i,
                counts.len(),
                top.join(", ")
            );
        }
        println!();
        println!(
            "Use --list for all unique values, --stats for counts, --search <FEATURE> to find entries."
        );
    }

    Ok(())
}
