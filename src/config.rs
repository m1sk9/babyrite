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
        static ENV_CONFIG: std::sync::OnceLock<EnvConfig> = std::sync::OnceLock::new();
        ENV_CONFIG
            .get_or_init(|| envy::from_env().expect("Failed to load environment configuration."))
    }
}

/// Babyrite configuration.
///
/// All configuration default values are `false`.
#[derive(Deserialize, Debug, Default)]
pub struct BabyriteConfig {
    /// If enabled, logs are output in JSON format.
    #[serde(default)]
    pub json_logging: bool,
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
}

/// Deserialize a string as an `Option<String>`, treating empty strings as `None`.
pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}
