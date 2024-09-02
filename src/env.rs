#[derive(serde::Deserialize)]
pub struct BabyriteEnv {
    pub config_file_path: String,
    pub discord_api_token: String,
}

/// Stores and retrieves Babyrite's environment variables in memory.
///
/// # Returns
/// * `BABYRITE_ENV`: A reference to the `BabyriteEnv` struct containing the environment variables.
pub fn babyrite_envs() -> &'static BabyriteEnv {
    static BABYRITE_ENV: std::sync::OnceLock<BabyriteEnv> = std::sync::OnceLock::new();
    BABYRITE_ENV.get_or_init(|| {
        envy::from_env()
            .expect("Failed to read environment variables. Do you set the environment variables?")
    })
}
