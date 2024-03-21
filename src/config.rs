use crate::error::Error;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

pub struct MPVClient;

impl MPVClient {
    pub fn new() -> Command {
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

pub struct ReqClient;

impl ReqClient {
    pub fn new() -> Client {
        let raw_proxy = Config::load().expect("获取自定义设置失败").proxy;
        let proxy = match raw_proxy {
            Some(proxy) => proxy,
            None => "".to_string(),
        };

        let client = if proxy.is_empty() {
            Client::builder().build().unwrap()
        } else {
            println!("正在使用代理访问: {}", proxy);
            let req_proxy = reqwest::Proxy::all(proxy).expect("设置代理失败");
            Client::builder().proxy(req_proxy).build().unwrap()
        };
        client
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "default_mpv")]
    pub mpv: String,
    pub proxy: Option<String>,
}

impl Config {
    // Load config file and retruns `Config`
    //
    // If config file doesn't exists, returns default value
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
    }
}

// The default value of `Config.mpv`
fn default_mpv() -> String {
    #[cfg(windows)]
    return "mpv.exe".to_string();
    #[cfg(unix)]
    return "mpv".to_string();
}
