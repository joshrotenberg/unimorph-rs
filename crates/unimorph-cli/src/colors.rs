//! Color output utilities.
//!
//! Respects NO_COLOR environment variable and terminal detection.

use std::io::IsTerminal;

use owo_colors::{OwoColorize, Style};

/// Check if color output should be enabled.
///
/// Colors are disabled if:
/// - NO_COLOR environment variable is set (any value)
/// - stdout is not a terminal (piped output)
pub fn should_colorize() -> bool {
    std::env::var("NO_COLOR").is_err() && std::io::stdout().is_terminal()
}

/// Style for language codes (e.g., "heb", "ita")
pub fn lang_style() -> Style {
    Style::new().cyan().bold()
}

/// Style for lemmas (dictionary forms)
pub fn lemma_style() -> Style {
    Style::new().green()
}

/// Style for surface forms
pub fn form_style() -> Style {
    Style::new().yellow()
}

/// Style for feature values
pub fn feature_style() -> Style {
    Style::new().magenta()
}

/// Style for numbers/counts
pub fn number_style() -> Style {
    Style::new().blue().bold()
}

/// Style for success messages
pub fn success_style() -> Style {
    Style::new().green().bold()
}

/// Style for warning messages
pub fn warning_style() -> Style {
    Style::new().yellow().bold()
}

/// Style for error messages
#[allow(dead_code)]
pub fn error_style() -> Style {
    Style::new().red().bold()
}

/// Style for dimmed/secondary text
pub fn dim_style() -> Style {
    Style::new().dimmed()
}

/// Style for headers/titles
pub fn header_style() -> Style {
    Style::new().bold()
}

/// Conditionally apply style based on whether colors are enabled.
pub fn styled<T: std::fmt::Display>(value: T, style: Style, colorize: bool) -> String {
    if colorize {
        value.style(style).to_string()
    } else {
        value.to_string()
    }
}

/// Helper macro for styled output that respects color settings.
#[macro_export]
macro_rules! colorize {
    ($value:expr, $style:expr, $enabled:expr) => {
        $crate::colors::styled($value, $style, $enabled)
    };
}
