use crate::error::Error;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_UA: &str = "Emby/3.2.32-17.32 (Linux;Android 13) ExoPlayerLib/2.13.2";

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
    pub mpv: String,
    pub proxy: Option<String>,
    pub useragent: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mpv: default_mpv(),
            proxy: None,
            useragent: Some(DEFAULT_UA.to_string()),
        }
    }
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

        Ok(Config::default())
    }
}

// 获取 config.toml 路径
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

// 设置 mpv 默认程序
fn default_mpv() -> String {
    #[cfg(windows)]
    return "mpv.exe".to_string();
    #[cfg(unix)]
    return "mpv".to_string();
}
