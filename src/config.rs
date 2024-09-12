use crate::error::Error;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

pub struct MPVClient;

impl MPVClient {
    pub fn build() -> Command {
        let mpv_command = Config::load().expect("获取自定义配置失败").mpv;

        match mpv_command.is_empty() {
            true => Command::new(default_mpv()),
            false => {
                println!("当前使用的MPV路径为: {}", mpv_command);
                Command::new(mpv_command)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_mpv")]
    pub mpv: String,
    pub proxy: Option<String>,
    pub useragent: Option<String>,
}

impl Config {
    // 读取config.toml配置信息
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
    #[cfg(windows)]
    let config_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");
    #[cfg(unix)]
    let config_path = dirs::home_dir()
        .unwrap()
        .join(".config/mpv-handler/config.toml");

    Ok(config_path)
}

// The defalut value of `Config`
fn default_config() -> Config {
    Config {
        mpv: default_mpv(),
        proxy: None,
        useragent: Some(DEFAULT_UA.to_string()),
    }
}

// The default value of `Config.mpv`
fn default_mpv() -> String {
    #[cfg(windows)]
    return "mpv.exe".to_string();
    #[cfg(unix)]
    return "mpv".to_string();
}
