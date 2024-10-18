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
    pub fn init() {
        match &babyrite_envs().config_file_path {
            Some(p) => {
                BABYRITE_CONFIG
                    .set(
                        toml::from_str(
                            &std::fs::read_to_string(p)
                                .expect("Failed to read config.yaml. Please check if the file exists."),
                        )
                            .expect("Failed to parse config.yaml. Please check if the settings are correct."),
                    )
                    .unwrap();
            }
            None => {
                BABYRITE_CONFIG.set(BabyriteConfig::default()).unwrap();
            }
        };
    }

    pub fn get() -> &'static BabyriteConfig {
        BABYRITE_CONFIG.get().unwrap()
    }
}
