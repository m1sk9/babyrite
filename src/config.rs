use serde::Deserialize;

use crate::babyrite_envs;

#[derive(Debug, Deserialize)]
pub struct BabyriteConfig {
    pub bypass_guilds: bool,
    pub citation_mention: bool,
    pub logger_format: LoggerFormat,
}

#[derive(Debug, Deserialize)]
pub enum LoggerFormat {
    #[serde(alias = "compact")]
    Compact,
    #[serde(alias = "json")]
    Json,
}

pub static BABYRITE_CONFIG: once_cell::sync::OnceCell<BabyriteConfig> =
    once_cell::sync::OnceCell::new();

impl BabyriteConfig {
    pub fn init() {
        BABYRITE_CONFIG
            .set(
                serde_yaml::from_reader(std::io::BufReader::new(
                    std::fs::File::open(&babyrite_envs().config_file_path)
                        .expect("Failed to open configuration file."),
                ))
                .expect("Failed to parse config.yaml. Please check if the settings are correct."),
            )
            .unwrap();
    }

    pub fn get() -> &'static BabyriteConfig {
        BABYRITE_CONFIG.get().unwrap()
    }
}