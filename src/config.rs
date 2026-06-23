//! Configuration module for Babyrite.
//!
//! This module handles both environment variables and file-based configuration.

use serde::Deserialize;
use std::sync::OnceLock;

/// Global configuration instance.
pub static CONFIG: OnceLock<BabyriteConfig> = OnceLock::new();

/// Environment variable configuration.
#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    /// Discord API token for bot authentication.
    pub discord_api_token: String,
    /// Optional path to the configuration file.
    #[serde(default)]
    #[serde(deserialize_with = "crate::config::empty_string_as_none")]
    pub config_file_path: Option<String>,
}

impl EnvConfig {
    /// Returns a reference to the environment configuration.
    ///
    /// Initializes the configuration from environment variables on first call.
    pub fn get() -> &'static EnvConfig {
        static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();
        ENV_CONFIG
            .get_or_init(|| envy::from_env().expect("Failed to load environment configuration."))
    }
}

/// Babyrite configuration.
///
/// Loaded from `config.toml`. All fields have default values, so existing
/// configuration files without the new sections will continue to work.
#[derive(Deserialize, Debug, Default)]
pub struct BabyriteConfig {
    /// If enabled, logs are output in JSON format.
    ///
    /// Deprecated: use `[log] format = "json"` instead. Kept for backward
    /// compatibility — it is only consulted when `log.format` is unset
    /// (see [`BabyriteConfig::resolved_log_format`]).
    #[serde(default)]
    pub json_logging: bool,
    /// Logging configuration (level and format).
    #[serde(default)]
    pub log: LogConfig,
    /// Feature flags for enabling/disabling specific functionality.
    #[serde(default)]
    pub features: FeatureConfig,
    /// GitHub-related configuration.
    #[serde(default)]
    pub github: GitHubConfig,
}

/// Logging configuration.
///
/// Controls the log level filter and output format.
#[derive(Deserialize, Debug)]
pub struct LogConfig {
    /// Tracing filter directive used when `RUST_LOG` is not set.
    ///
    /// Accepts the same syntax as `RUST_LOG` (e.g. `babyrite=debug`).
    /// Defaults to `babyrite=info`.
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Output format. When unset, falls back to the deprecated `json_logging`
    /// flag (see [`BabyriteConfig::resolved_log_format`]).
    #[serde(default)]
    pub format: Option<LogFormat>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: None,
        }
    }
}

/// Log output format.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable compact format.
    Compact,
    /// Structured JSON format (recommended for Grafana Loki and similar).
    Json,
}

/// Feature flags configuration.
///
/// Controls which link expansion features are enabled.
#[derive(Deserialize, Debug)]
pub struct FeatureConfig {
    /// Whether GitHub Permalink expansion is enabled.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_true")]
    pub github_permalink: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            github_permalink: default_true(),
        }
    }
}

/// GitHub-related configuration.
#[derive(Deserialize, Debug)]
pub struct GitHubConfig {
    /// Maximum number of lines to display without truncation.
    ///
    /// Defaults to `50`.
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_max_lines() -> usize {
    50
}

fn default_log_level() -> String {
    "babyrite=info".to_string()
}

/// Errors that can occur when loading configuration.
#[derive(thiserror::Error, Debug)]
pub enum BabyriteConfigError {
    /// Failed to read the configuration file from disk.
    #[error("Failed to read configuration file.")]
    Read,
    /// Failed to parse the configuration file contents.
    #[error("Failed to parse configuration file.")]
    Parse,
    /// Failed to set the global configuration.
    #[error("Failed to set configuration file.")]
    Set,
}

impl BabyriteConfig {
    /// Initializes the global configuration.
    ///
    /// Loads configuration from a file if `CONFIG_FILE_PATH` is set,
    /// otherwise uses default values.
    pub fn init() -> anyhow::Result<(), BabyriteConfigError> {
        let envs = EnvConfig::get();
        match &envs.config_file_path {
            Some(p) => {
                let buffer = &std::fs::read_to_string(p).map_err(|_| BabyriteConfigError::Read)?;
                let config: BabyriteConfig =
                    toml::from_str(buffer).map_err(|_| BabyriteConfigError::Parse)?;
                Ok(CONFIG.set(config).map_err(|_| BabyriteConfigError::Parse)?)
            }
            None => Ok(CONFIG
                .set(BabyriteConfig::default())
                .map_err(|_| BabyriteConfigError::Set)?),
        }
    }

    /// Returns a reference to the global configuration.
    ///
    /// # Panics
    ///
    /// Panics if [`BabyriteConfig::init`] has not been called.
    pub fn get() -> &'static BabyriteConfig {
        CONFIG.get().expect("Failed to get configuration.")
    }

    /// Resolves the effective log output format.
    ///
    /// Uses `[log] format` when set; otherwise falls back to the deprecated
    /// `json_logging` flag (`true` → JSON, `false` → compact) so existing
    /// configuration files keep their previous behavior.
    pub fn resolved_log_format(&self) -> LogFormat {
        self.log.format.unwrap_or({
            if self.json_logging {
                LogFormat::Json
            } else {
                LogFormat::Compact
            }
        })
    }
}

/// Deserialize a string as an `Option<String>`, treating empty strings as `None`.
pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = BabyriteConfig::default();
        assert!(!config.json_logging);
        assert_eq!(config.log.level, "babyrite=info");
        assert_eq!(config.log.format, None);
        assert_eq!(config.resolved_log_format(), LogFormat::Compact);
        assert!(config.features.github_permalink);
        assert_eq!(config.github.max_lines, 50);
    }

    #[test]
    fn deserialize_empty_config() {
        let config: BabyriteConfig = toml::from_str("").unwrap();
        assert!(!config.json_logging);
        assert_eq!(config.log.level, "babyrite=info");
        assert_eq!(config.log.format, None);
        assert_eq!(config.resolved_log_format(), LogFormat::Compact);
        assert!(config.features.github_permalink);
        assert_eq!(config.github.max_lines, 50);
    }

    #[test]
    fn deserialize_log_section() {
        let toml_str = r#"
            [log]
            level = "babyrite=debug"
            format = "json"
        "#;
        let config: BabyriteConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.log.level, "babyrite=debug");
        assert_eq!(config.log.format, Some(LogFormat::Json));
        assert_eq!(config.resolved_log_format(), LogFormat::Json);
    }

    #[test]
    fn resolved_log_format_falls_back_to_json_logging() {
        // `[log] format` unset but the deprecated `json_logging` is enabled:
        // the format should resolve to JSON for backward compatibility.
        let toml_str = r#"
            json_logging = true
        "#;
        let config: BabyriteConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.log.format, None);
        assert_eq!(config.resolved_log_format(), LogFormat::Json);
    }

    #[test]
    fn log_format_overrides_json_logging() {
        // When both are set, `[log] format` wins over the deprecated flag.
        let toml_str = r#"
            json_logging = true

            [log]
            format = "compact"
        "#;
        let config: BabyriteConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.resolved_log_format(), LogFormat::Compact);
    }

    #[test]
    fn deserialize_full_config() {
        let toml_str = r#"
            json_logging = true

            [features]
            github_permalink = false

            [github]
            max_lines = 100
        "#;
        let config: BabyriteConfig = toml::from_str(toml_str).unwrap();
        assert!(config.json_logging);
        assert!(!config.features.github_permalink);
        assert_eq!(config.github.max_lines, 100);
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = r#"
            json_logging = true
        "#;
        let config: BabyriteConfig = toml::from_str(toml_str).unwrap();
        assert!(config.json_logging);
        // defaults
        assert!(config.features.github_permalink);
        assert_eq!(config.github.max_lines, 50);
    }

    #[test]
    fn empty_string_as_none_with_empty() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "empty_string_as_none")]
            value: Option<String>,
        }

        let t: Test = toml::from_str(r#"value = """#).unwrap();
        assert!(t.value.is_none());
    }

    #[test]
    fn empty_string_as_none_with_value() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "empty_string_as_none")]
            value: Option<String>,
        }

        let t: Test = toml::from_str(r#"value = "hello""#).unwrap();
        assert_eq!(t.value.as_deref(), Some("hello"));
    }

    #[test]
    fn empty_string_as_none_absent() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "empty_string_as_none")]
            value: Option<String>,
        }

        let t: Test = toml::from_str("").unwrap();
        assert!(t.value.is_none());
    }
}
