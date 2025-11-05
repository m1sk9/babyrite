use serde::Deserialize;

pub static CONFIG: once_cell::sync::OnceCell<BabyriteConfig> = once_cell::sync::OnceCell::new();

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub discord_api_token: String,
    #[serde(default)]
    #[serde(deserialize_with = "crate::config::empty_string_as_none")]
    pub config_file_path: Option<String>,
}

impl EnvConfig {
    pub fn get() -> &'static EnvConfig {
        static ENV_CONFIG: std::sync::OnceLock<EnvConfig> = std::sync::OnceLock::new();
        ENV_CONFIG
            .get_or_init(|| envy::from_env().expect("Failed to load environment configuration."))
    }
}

// TODO: Support for the old values will be removed in v0.20.0.
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
#[derive(Deserialize, Debug)]
pub struct BabyriteConfig {
    // If enabled, previews are generated with mentions.
    #[serde(alias = "is_mention", default = "default_true")]
    pub preview_mention: bool,
    // If enabled, preview can be deleted.
    #[serde(alias = "is_deletable", default = "default_true")]
    pub preview_deletion: bool,
    // If enabled, allow preview generation of NSFW content.
    #[serde(alias = "is_allow_nsfw", default = "default_false")]
    pub allow_nsfw: bool,
    // If enabled, logs are output in JSON format.
    #[serde(default)]
    pub json_logging: bool,
}

impl Default for BabyriteConfig {
    fn default() -> Self {
        Self {
            preview_mention: true,
            preview_deletion: true,
            allow_nsfw: false,
            json_logging: false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BabyriteConfigError {
    #[error("Failed to read configuration file.")]
    FailedToReadConfig,
    #[error("Failed to parse configuration file.")]
    FailedToParseConfig,
    #[error("Failed to set configuration file.")]
    FailedToSetConfig,
}

impl BabyriteConfig {
    pub fn init() -> anyhow::Result<(), BabyriteConfigError> {
        let envs = EnvConfig::get();
        match &envs.config_file_path {
            Some(p) => {
                let buffer = &std::fs::read_to_string(p)
                    .map_err(|_| BabyriteConfigError::FailedToReadConfig)?;
                let config: BabyriteConfig =
                    toml::from_str(buffer).map_err(|_| BabyriteConfigError::FailedToParseConfig)?;
                Ok(CONFIG
                    .set(config)
                    .map_err(|_| BabyriteConfigError::FailedToParseConfig)?)
            }
            None => Ok(CONFIG
                .set(BabyriteConfig::default())
                .map_err(|_| BabyriteConfigError::FailedToSetConfig)?),
        }
    }

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
