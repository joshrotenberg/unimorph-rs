//! Config command implementation.

use std::fs;

use color_eyre::eyre::{Context, Result};

use crate::colors::{dim_style, header_style, lang_style, should_colorize, styled, success_style};
use crate::config::Config;

/// Show the current configuration.
pub fn cmd_config_show(json: bool) -> Result<()> {
    let config = Config::load();
    let config_path = Config::config_path();

    if json {
        #[derive(serde::Serialize)]
        struct ConfigInfo {
            path: Option<String>,
            exists: bool,
            config: ConfigJson,
        }

        #[derive(serde::Serialize)]
        struct ConfigJson {
            default_lang: Option<String>,
            data_dir: Option<String>,
            output_format: Option<String>,
            http_proxy: Option<String>,
            no_color: Option<bool>,
            languages: std::collections::HashMap<String, LanguageJson>,
        }

        #[derive(serde::Serialize)]
        struct LanguageJson {
            alias: Option<String>,
        }

        let languages = config
            .languages
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    LanguageJson {
                        alias: v.alias.clone(),
                    },
                )
            })
            .collect();

        let info = ConfigInfo {
            path: config_path.as_ref().map(|p| p.display().to_string()),
            exists: config_path.as_ref().is_some_and(|p| p.exists()),
            config: ConfigJson {
                default_lang: config.default_lang,
                data_dir: config.data_dir.map(|p| p.display().to_string()),
                output_format: config.output_format,
                http_proxy: config.http_proxy,
                no_color: config.no_color,
                languages,
            },
        };

        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        let colorize = should_colorize();

        println!("{}", styled("Configuration", header_style(), colorize));
        println!();

        // Show config file path and status
        match &config_path {
            Some(path) => {
                let exists = path.exists();
                println!(
                    "  {} {}",
                    styled("Path:", dim_style(), colorize),
                    path.display()
                );
                println!(
                    "  {} {}",
                    styled("Status:", dim_style(), colorize),
                    if exists { "exists" } else { "not created" }
                );
            }
            None => {
                println!(
                    "  {} could not determine config directory",
                    styled("Path:", dim_style(), colorize)
                );
            }
        }

        println!();
        println!("{}", styled("Current Settings", header_style(), colorize));
        println!();

        // Default language
        println!(
            "  {} {}",
            styled("default_lang:", dim_style(), colorize),
            config
                .default_lang
                .as_deref()
                .map(|l| styled(l, lang_style(), colorize))
                .unwrap_or_else(|| "(not set)".to_string())
        );

        // Data directory
        println!(
            "  {} {}",
            styled("data_dir:", dim_style(), colorize),
            config
                .data_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(default)".to_string())
        );

        // Output format
        println!(
            "  {} {}",
            styled("output_format:", dim_style(), colorize),
            config
                .output_format
                .as_deref()
                .unwrap_or("(default: table)")
        );

        // No color
        println!(
            "  {} {}",
            styled("no_color:", dim_style(), colorize),
            config
                .no_color
                .map(|b| b.to_string())
                .unwrap_or_else(|| "(not set)".to_string())
        );

        // Language aliases
        if !config.languages.is_empty() {
            println!();
            println!("{}", styled("Language Aliases", header_style(), colorize));
            println!();

            for (code, lang_config) in &config.languages {
                if let Some(alias) = &lang_config.alias {
                    println!(
                        "  {} -> {}",
                        styled(alias, dim_style(), colorize),
                        styled(code, lang_style(), colorize)
                    );
                }
            }
        }
    }

    Ok(())
}

/// Initialize a new configuration file with example content.
pub fn cmd_config_init(force: bool, json: bool) -> Result<()> {
    let config_path = Config::config_path()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

    if config_path.exists() && !force {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": "Config file already exists. Use --force to overwrite.",
                    "path": config_path.display().to_string()
                })
            );
        } else {
            println!(
                "Config file already exists at: {}\n\nUse --force to overwrite.",
                config_path.display()
            );
        }
        return Ok(());
    }

    // Create parent directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let example_config = r#"# UniMorph CLI Configuration
# See: https://github.com/joshrotenberg/unimorph-rs

# Default language for commands (can be overridden with positional arg)
# default_lang = "heb"

# Data directory (can be overridden with --data-dir or UNIMORPH_DATA)
# data_dir = "~/.cache/unimorph"

# Default output format for commands that support it
# output_format = "table"  # or "json"

# HTTP proxy for GitHub API requests
# http_proxy = "http://proxy.example.com:8080"

# Disable colors (alternative to NO_COLOR env var)
# no_color = false

# Per-language settings
# Define aliases to use friendly names instead of ISO 639-3 codes

# [languages.heb]
# alias = "hebrew"

# [languages.ita]
# alias = "italian"

# [languages.deu]
# alias = "german"

# [languages.spa]
# alias = "spanish"

# [languages.fra]
# alias = "french"
"#;

    fs::write(&config_path, example_config).context("Failed to write config file")?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "path": config_path.display().to_string()
            })
        );
    } else {
        let colorize = should_colorize();
        println!(
            "{} Created config file at: {}",
            styled("Success!", success_style(), colorize),
            config_path.display()
        );
    }

    Ok(())
}

/// Show the path to the config file.
pub fn cmd_config_path(json: bool) -> Result<()> {
    let config_path = Config::config_path();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "path": config_path.as_ref().map(|p| p.display().to_string()),
                "exists": config_path.as_ref().is_some_and(|p| p.exists())
            })
        );
    } else {
        match config_path {
            Some(path) => {
                println!("{}", path.display());
            }
            None => {
                println!("Could not determine config directory");
            }
        }
    }

    Ok(())
}
