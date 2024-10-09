use serde::Deserialize;

use crate::babyrite_envs;

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
    pub bypass_guild_check: bool,
    pub is_mention: bool,
}

pub static BABYRITE_CONFIG: once_cell::sync::OnceCell<BabyriteConfig> =
    once_cell::sync::OnceCell::new();

impl BabyriteConfig {
    pub fn init() {
        BABYRITE_CONFIG
            .set(
                toml::from_str(
                    &std::fs::read_to_string(&babyrite_envs().config_file_path)
                        .expect("Failed to read config.yaml. Please check if the file exists."),
                )
                .expect("Failed to parse config.yaml. Please check if the settings are correct."),
            )
            .unwrap();
    }

    pub fn get() -> &'static BabyriteConfig {
        BABYRITE_CONFIG.get().unwrap()
    }
}
