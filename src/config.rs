use crate::error::Error;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_mpv")]
    pub mpv: String,
}

impl Config {
    /// Load config file and retruns `Config`
    ///
    /// If config file doesn't exists, returns default value
    pub fn load() -> Result<Config, Error> {
        let path = config_path()?;

        if path.exists() {
            let data: String = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&data)?;

            return Ok(config);
        }

        Ok(default_config())
    }
}

// Get config file path
fn config_path() -> Result<PathBuf, Error> {
    let config_path = env::current_exe()?.parent().unwrap().join("config.toml");

    Ok(config_path)
}

/// The defalut value of `Config`
fn default_config() -> Config {
    Config { mpv: default_mpv() }
}

/// The default value of `Config.mpv`
fn default_mpv() -> String {
    "mpv.exe".to_string()
}
