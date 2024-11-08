use crate::babyrite_envs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BabyriteConfig {
    pub logger_format: LoggerFormat,
    pub preview: PreviewConfig,
}

#[derive(Debug, Deserialize)]
pub enum LoggerFormat {
    #[serde(alias = "compact")]
    Compact,
    #[serde(alias = "json")]
    Json,
}

#[derive(Debug, Deserialize)]
pub struct PreviewConfig {
    pub is_mention: bool,
}

pub static BABYRITE_CONFIG: once_cell::sync::OnceCell<BabyriteConfig> =
    once_cell::sync::OnceCell::new();

impl Default for BabyriteConfig {
    fn default() -> Self {
        Self {
            logger_format: LoggerFormat::Compact,
            preview: PreviewConfig { is_mention: true },
        }
    }
}

impl BabyriteConfig {
    /// Initializes the Babyrite configuration.
    ///
    /// This function attempts to read the configuration from a file specified
    /// in the environment variables. If the file is not found or cannot be read,
    /// it falls back to the default configuration.
    ///
    /// # Panics
    ///
    /// This function will panic if the configuration file is specified but cannot
    /// be read or parsed correctly.
    pub fn init() {
        match &babyrite_envs().config_file_path {
            Some(p) => {
                BABYRITE_CONFIG
                        .set(
                            toml::from_str(
                                &std::fs::read_to_string(p)
                                    .expect("Failed to read config.toml. Please check if the file exists."),
                            )
                                .expect("Failed to parse config.toml. Please check if the settings are correct."),
                        )
                        .unwrap();
            }
            None => {
                BABYRITE_CONFIG.set(BabyriteConfig::default()).unwrap();
            }
        };
    }

    /// Retrieves the Babyrite configuration.
    ///
    /// This function returns a reference to the Babyrite configuration that has been
    /// initialized. It will panic if the configuration has not been set.
    ///
    /// # Panics
    ///
    /// This function will panic if the configuration has not been initialized.
    /// Use `BabyriteConfig::init` to initialize the configuration.
    ///
    /// # Examples
    ///
    /// ```rs
    /// let config = BabyriteConfig::get();
    ///
    /// assert_eq!(config.logger_format, LoggerFormat::Compact);
    /// ```
    pub fn get() -> &'static BabyriteConfig {
        BABYRITE_CONFIG.get().unwrap()
    }
}
