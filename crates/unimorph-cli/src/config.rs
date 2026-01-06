//! Configuration file support for unimorph CLI.
//!
//! Configuration is loaded from `~/.config/unimorph/config.toml` on all platforms.
//! This is consistent with the data directory at `~/.cache/unimorph/`.
//!
//! Priority order (highest to lowest):
//! 1. Command-line flags
//! 2. Environment variables
//! 3. Configuration file
//! 4. Built-in defaults

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::{debug, warn};

/// Language-specific configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LanguageConfig {
    /// Alias for this language (e.g., "hebrew" for "heb").
    pub alias: Option<String>,
}

/// Main configuration structure.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default language for commands that accept a language argument.
    pub default_lang: Option<String>,

    /// Data directory for storing downloaded datasets.
    /// Can be overridden with --data-dir or UNIMORPH_DATA env var.
    pub data_dir: Option<PathBuf>,

    /// Default output format for commands that support it.
    /// Valid values: "table", "json"
    pub output_format: Option<String>,

    /// HTTP proxy for GitHub API requests.
    pub http_proxy: Option<String>,

    /// Disable colored output (alternative to NO_COLOR env var).
    pub no_color: Option<bool>,

    /// Per-language settings.
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,
}

impl Config {
    /// Load configuration from the default config file location.
    ///
    /// Returns a default config if the file doesn't exist.
    /// Logs a warning if the file exists but cannot be parsed.
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) => Self::load_from(&path),
            None => {
                debug!("Could not determine config directory");
                Self::default()
            }
        }
    }

    /// Load configuration from a specific path.
    pub fn load_from(path: &Path) -> Self {
        if !path.exists() {
            debug!(path = %path.display(), "Config file not found, using defaults");
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    debug!(path = %path.display(), "Loaded config file");
                    config
                }
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "Failed to parse config file");
                    Self::default()
                }
            },
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to read config file");
                Self::default()
            }
        }
    }

    /// Get the default config file path.
    ///
    /// Always uses `~/.config/unimorph/config.toml` for consistency with
    /// the data directory at `~/.cache/unimorph/`.
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(".config").join("unimorph").join("config.toml"))
    }

    /// Resolve a language code, checking aliases first.
    ///
    /// If the input matches a language alias, returns the corresponding
    /// language code. Otherwise, returns the input unchanged.
    pub fn resolve_lang(&self, lang_or_alias: &str) -> String {
        // Check if any language has this alias
        for (code, lang_config) in &self.languages {
            if let Some(alias) = &lang_config.alias
                && alias.eq_ignore_ascii_case(lang_or_alias)
            {
                debug!(
                    alias = lang_or_alias,
                    code = code,
                    "Resolved language alias"
                );
                return code.clone();
            }
        }
        lang_or_alias.to_string()
    }

    /// Get the default language from environment variable or config file.
    ///
    /// Priority order:
    /// 1. UNIMORPH_LANG environment variable
    /// 2. Config file default_lang
    ///
    /// Returns None if no default is set.
    pub fn default_lang(&self) -> Option<String> {
        // Check environment variable first
        if let Ok(lang) = std::env::var("UNIMORPH_LANG")
            && !lang.is_empty()
        {
            debug!(lang = %lang, "Using default language from UNIMORPH_LANG");
            return Some(self.resolve_lang(&lang));
        }

        // Fall back to config file
        if let Some(lang) = &self.default_lang {
            debug!(lang = %lang, "Using default language from config file");
            return Some(self.resolve_lang(lang));
        }

        None
    }

    /// Resolve the effective language: use provided value or fall back to default.
    ///
    /// If `lang` is Some, resolves any aliases and returns it.
    /// If `lang` is None, returns the default language (env > config).
    /// Returns None if no language is provided and no default is set.
    pub fn effective_lang(&self, lang: Option<&str>) -> Option<String> {
        match lang {
            Some(l) => Some(self.resolve_lang(l)),
            None => self.default_lang(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.default_lang.is_none());
        assert!(config.data_dir.is_none());
        assert!(config.languages.is_empty());
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
            default_lang = "heb"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.default_lang, Some("heb".to_string()));
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
            default_lang = "heb"
            data_dir = "~/.cache/unimorph"
            output_format = "table"
            no_color = false

            [languages.heb]
            alias = "hebrew"

            [languages.ita]
            alias = "italian"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.default_lang, Some("heb".to_string()));
        assert_eq!(config.data_dir, Some(PathBuf::from("~/.cache/unimorph")));
        assert_eq!(config.output_format, Some("table".to_string()));
        assert_eq!(config.no_color, Some(false));
        assert_eq!(config.languages.len(), 2);
        assert_eq!(
            config.languages.get("heb").unwrap().alias,
            Some("hebrew".to_string())
        );
    }

    #[test]
    fn test_resolve_lang_with_alias() {
        let toml = r#"
            [languages.heb]
            alias = "hebrew"

            [languages.ita]
            alias = "italian"
        "#;
        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.resolve_lang("hebrew"), "heb");
        assert_eq!(config.resolve_lang("Hebrew"), "heb"); // Case insensitive
        assert_eq!(config.resolve_lang("italian"), "ita");
        assert_eq!(config.resolve_lang("deu"), "deu"); // No alias, pass through
    }

    #[test]
    fn test_load_nonexistent_file() {
        let config = Config::load_from(Path::new("/nonexistent/path/config.toml"));
        assert!(config.default_lang.is_none());
    }

    #[test]
    fn test_effective_lang_with_explicit() {
        let toml = r#"
            default_lang = "ita"
        "#;
        let config: Config = toml::from_str(toml).unwrap();

        // Explicit lang should always be used regardless of default
        assert_eq!(config.effective_lang(Some("heb")), Some("heb".to_string()));
    }

    #[test]
    fn test_effective_lang_resolves_alias() {
        let toml = r#"
            [languages.heb]
            alias = "hebrew"
        "#;
        let config: Config = toml::from_str(toml).unwrap();

        // Alias should be resolved
        assert_eq!(
            config.effective_lang(Some("hebrew")),
            Some("heb".to_string())
        );
    }
}
