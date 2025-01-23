use crate::get_env_config;

pub static CONFIG: once_cell::sync::OnceCell<PreviewConfig> = once_cell::sync::OnceCell::new();

#[derive(serde::Deserialize, Debug)]
pub struct PreviewConfig {
    // Enable optional features.
    pub feature_flag: Option<String>,
    // If enabled, previews are generated with mentions.
    pub is_mention: bool,
    // If enabled, preview can be deleted.
    pub is_deletable: bool,
    // If enabled, allow preview generation of NSFW content.
    pub is_allow_nsfw: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            feature_flag: None,
            is_mention: true,
            is_deletable: true,
            is_allow_nsfw: false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PreviewConfigError {
    #[error("Failed to read configuration file.")]
    FailedToReadConfig,
    #[error("Failed to parse configuration file.")]
    FailedToParseConfig,
}

impl PreviewConfig {
    pub fn init() -> anyhow::Result<(), PreviewConfigError> {
        let envs = get_env_config();
        match &envs.config_file_path {
            Some(p) => {
                let buffer = &std::fs::read_to_string(p)
                    .map_err(|_| PreviewConfigError::FailedToReadConfig)?;
                let config: PreviewConfig =
                    toml::from_str(buffer).map_err(|_| PreviewConfigError::FailedToParseConfig)?;
                Ok(CONFIG
                    .set(config)
                    .map_err(|_| PreviewConfigError::FailedToParseConfig)?)
            }
            None => Ok(CONFIG
                .set(PreviewConfig::default())
                .map_err(|_| PreviewConfigError::FailedToParseConfig)?),
        }
    }

    pub fn get_config() -> &'static PreviewConfig {
        CONFIG.get().expect("Failed to get configuration.")
    }

    pub fn get_feature_flag(flag: &str) -> bool {
        let c = Self::get_config();
        c.feature_flag.as_ref().is_some_and(|f| f.contains(flag))
    }
}
