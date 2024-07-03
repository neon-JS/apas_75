use super::error::ConfigError;
use crate::optionize;
use std::fmt::{Display, Formatter};

pub const DEFAULT_INDICATOR_THEME_START: &str = "### apas_75 theme start ###";
pub const DEFAULT_INDICATOR_THEME_END: &str = "### apas_75 theme end ###";

/// Representation of a TOML config file that may contains a custom theme.
pub struct Config {
    /// Static config part that appears before `theme`.
    /// Is `None` if config file starts with theme.
    static_first: Option<String>,
    /// Static config part that appears after `theme`.
    /// Is `None` if config file ends with theme.
    static_second: Option<String>,
    /// Theme config.
    /// Is `None` if not set.
    theme: Option<String>,
    indicator_theme_start: String,
    indicator_theme_end: String,
}

impl Config {
    /// Tries building a `Config` based on content of a TOML file
    ///
    /// # Arguments
    ///
    /// * `file_content` - Content of TOML file
    /// * `indicator_theme_start` - Indicator (comment) for the start of the
    ///    theme part. See `DEFAULT_INDICATOR_THEME_START`.
    /// * `indicator_theme_end` - Indicator (comment) for the end of the
    ///    theme part. See `DEFAULT_INDICATOR_THEME_END`.
    ///
    /// # Returns
    ///
    /// `Ok(Config)` or `Err` if the file cannot be split into default config and
    /// theme part correctly. May happen if file is corrupted.
    pub fn new(
        file_content: &str,
        indicator_theme_start: &str,
        indicator_theme_end: &str,
    ) -> Result<Config, ConfigError> {
        match (
            get_single_index(file_content, indicator_theme_start),
            get_single_index(file_content, indicator_theme_end),
        ) {
            (Some(theme_start), Some(theme_end)) => {
                if theme_start > theme_end {
                    return Err(ConfigError::ThemeIndicatorsWrongOrder);
                }

                let theme_content_start = theme_start + indicator_theme_start.len();
                let static_second_start = theme_end + indicator_theme_end.len();

                Ok(Config {
                    static_first: optionize!(file_content[0..theme_start]),
                    theme: optionize!(file_content[theme_content_start..theme_end]),
                    static_second: optionize!(file_content[static_second_start..]),
                    indicator_theme_start: String::from(indicator_theme_start),
                    indicator_theme_end: String::from(indicator_theme_end),
                })
            }
            (None, None) => Ok(Config {
                static_first: Some(String::from(file_content)),
                static_second: None,
                theme: None,
                indicator_theme_start: String::from(indicator_theme_start),
                indicator_theme_end: String::from(indicator_theme_end),
            }),
            _ => Err(ConfigError::MissingThemeIndicator),
        }
    }

    /// Overrides `theme` part of `Config`
    ///
    /// # Arguments
    ///
    /// * `theme` - Theme data
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = Some(String::from(theme.trim()));
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let static_first = match &self.static_first {
            Some(s) => s.clone(),
            None => String::new(),
        };
        let static_second = match &self.static_second {
            Some(s) => s.clone(),
            None => String::new(),
        };
        let theme = match &self.theme {
            Some(s) => s.clone(),
            None => String::new(),
        };

        let document = format!(
            "{static_first}

{}

{theme}

{}

{static_second}",
            &self.indicator_theme_start, &self.indicator_theme_end
        );

        write!(f, "{}", document.trim())
    }
}

#[cfg(test)]
mod tests_config {
    use super::*;

    #[test]
    fn new_works() {
        let toml = "### apas_75 theme start ###\ncustom\nlines\n### apas_75 theme end ###\n\ndefault\nlines";
        let config_result = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        assert!(config_result.is_ok());
    }

    #[test]
    fn split_works_on_start() {
        let toml = "### apas_75 theme start ###\ncustom\nlines\n### apas_75 theme end ###\n\ndefault\nlines";
        let config_result = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        assert!(config.static_first.is_none());
        assert!(config.static_second.is_some_and(|v| v == "default\nlines"));
        assert!(config.theme.is_some_and(|v| v == "custom\nlines"));
    }

    #[test]
    fn split_works_in_middle() {
        let toml = "default\nlines\nstart\n\n### apas_75 theme start ###\ncustom\nlines\n### apas_75 theme end ###\ndefault\nlines\nend";
        let config_result: Result<Config, ConfigError> = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        assert!(config
            .static_first
            .is_some_and(|v| v == "default\nlines\nstart"));
        assert!(config
            .static_second
            .is_some_and(|v| v == "default\nlines\nend"));
        assert!(config.theme.is_some_and(|v| v == "custom\nlines"));
    }

    #[test]
    fn split_works_on_end() {
        let toml = "default\nlines\nstart\ndefault\nlines\nend\n\n### apas_75 theme start ###\ncustom\nlines\n### apas_75 theme end ###";
        let config_result = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        assert!(config
            .static_first
            .is_some_and(|v| v == "default\nlines\nstart\ndefault\nlines\nend"));
        assert!(config.static_second.is_none());
        assert!(config.theme.is_some_and(|v| v == "custom\nlines"));
    }

    #[test]
    fn split_works_without_theme() {
        let toml = "default\nlines\nstart\ndefault\nlines\nend";
        let config_result = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        assert!(config
            .static_first
            .is_some_and(|v| v == "default\nlines\nstart\ndefault\nlines\nend"));
        assert!(config.static_second.is_none());
        assert!(config.theme.is_none());
    }

    #[test]
    fn to_string_works() {
        let toml = "default\nlines\nstart\n\n### apas_75 theme start ###\n### apas_75 theme end ###\ndefault\nlines\nend";
        let config_result: Result<Config, ConfigError> = Config::new(
            toml,
            DEFAULT_INDICATOR_THEME_START,
            DEFAULT_INDICATOR_THEME_END,
        );

        let mut config = config_result.unwrap();

        config.set_theme("custom\nlines");

        assert_eq!(
            config.to_string(),
            "default\nlines\nstart\n\n### apas_75 theme start ###\n\ncustom\nlines\n\n### apas_75 theme end ###\n\ndefault\nlines\nend"
        );
    }
}

/// Tries getting the **only** index of a `needle` in a `haystack`.
/// If `haystack` contains `needle` not exactly **one** time, `None` is returned.
///
/// # Arguments
///
/// * `haystack` - String to search for `needle`
/// * `needle` - The string that should be searched for in `haystack`
///
/// # Returns
///
/// Index of the **only** substring in `haystack` or `None`.
///
/// # Examples
///
/// ```
/// assert_eq!(get_single_index("Starships were meant to fly", "a"), None); // because "a" exists more than once
/// assert_eq!(get_single_index("Starships were meant to fly", "were"), Some(10));
/// ```
fn get_single_index(haystack: &str, needle: &str) -> Option<usize> {
    let matches = haystack
        .match_indices(needle)
        .take(2)
        .map(|(index, _)| index)
        .collect::<Vec<usize>>();

    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

#[cfg(test)]
mod tests_get_single_index {
    use super::*;

    #[test]
    fn works() {
        assert_eq!(get_single_index("Starships were meant to fly", "a"), None);

        assert_eq!(
            get_single_index("Starships were meant to fly", "Hands up"),
            None
        );

        assert_eq!(
            get_single_index("Starships were meant to fly", "were"),
            Some(10)
        );
    }
}

#[macro_export]
/// Converts given ` &str` into `Option<String>`. If `&str` is empty or whitespace-only, it will result in `None`.
/// Otherwise, `Some(&str.trim())` will result.
///
/// # Arguments
///
/// * `&str` - The string to convert
///
/// # Examples
///
/// ```
/// assert!(optionize!("").is_none());
/// assert!(optionize!(" this is a test ")
///     .is_some_and(|trimmed| trimmed == "this is a test"));
/// ```
macro_rules! optionize {
    ( $str:expr ) => {{
        let trimmed = $str.trim();

        if trimmed != "" {
            Some(String::from(trimmed))
        } else {
            None
        }
    }};
}

#[cfg(test)]
mod tests_optionize {
    use super::*;

    #[test]
    fn works() {
        assert!(optionize!("").is_none());

        assert!(optionize!(" this is a test ").is_some_and(|trimmed| trimmed == "this is a test"));
    }
}