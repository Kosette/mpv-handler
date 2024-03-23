use crate::error::Error;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;
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

pub struct Extractor;

impl Extractor {
    pub fn extract_urls(mpv_url: &str) -> Result<(String, String), String> {
        let url = mpv_url
            .trim_end_matches('=')
            .strip_prefix("mpv://play/")
            .unwrap();

        if url.contains("/?subfile=") {
            let parts: Vec<&str> = url.splitn(2, "/?subfile=").collect();
            let video_url = URL_SAFE_NO_PAD.decode(&parts[0]).unwrap();
            let subfile_url = URL_SAFE_NO_PAD.decode(&parts[1]).unwrap();

            Ok((
                String::from_utf8(video_url).unwrap(),
                String::from_utf8(subfile_url).unwrap(),
            ))
        } else {
            let video_url = URL_SAFE_NO_PAD.decode(url).unwrap();
            Ok((String::from_utf8(video_url).unwrap(), String::new()))
        }
    }

    pub fn extract_params(video_url: &str) -> (String, String, String, String) {
        // 定义正则表达式模式
        let pattern = Regex::new(
            r"^(https?://[^/]+)/emby/videos/(\d+)/.*?api_key=([^&]+).*?MediaSourceId=([^&]+)",
        )
        .unwrap();

        // 匹配并提取值
        if let Some(captures) = pattern.captures(video_url) {
            let host = &captures[1];
            let item_id = &captures[2];
            let api_key = &captures[3];
            let media_source_id = &captures[4];

            let result = (
                String::from(host),
                String::from(item_id),
                String::from(api_key),
                String::from(media_source_id),
            );
            return result;
        } else {
            return (String::new(), String::new(), String::new(), String::new());
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

    pub fn update_progress(ticks: u128, host: &str, item_id: &str, api_key: &str, media_id: &str) {
        let mut headers = HeaderMap::new();

        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key).unwrap());
        headers.insert(
            "X-Emby-Device-Id",
            HeaderValue::from_static("8b1e6f78-4965-423a-8a11-73e50882bcb3"),
        );
        headers.insert(
            "X-Emby-Device-Name",
            HeaderValue::from_static("Google Chrome"),
        );

        let stopped_body = json!({"ItemId":item_id,"MediaSourceId":media_id,"PositionTicks":ticks});

        let res = ReqClient::new()
            .post(&format!("{}/emby/Sessions/Playing/Stopped", host))
            .headers(headers)
            .json(&stopped_body)
            .send();

        match res {
            Ok(res) => {
                println!("正在回传进度，请求状态: {}", res.status());
            }
            Err(_) => println!("回传进度失败"),
        }
    }

    pub fn playing_status(host: &str, item_id: &str, api_key: &str, media_id: &str, switch: bool) {
        let client = ReqClient::new();

        let mut headers = HeaderMap::new();

        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key).unwrap());
        headers.insert(
            "X-Emby-Device-Id",
            HeaderValue::from_static("8b1e6f78-4965-423a-8a11-73e50882bcb3"),
        );
        headers.insert(
            "X-Emby-Device-Name",
            HeaderValue::from_static("Google Chrome"),
        );

        let playing_body = json!({"ItemId":item_id,"MediaSourceId":media_id});

        let url = match switch {
            true => format!("{}/emby/Sessions/Playing", host),
            false => format!("{}/emby/Sessions/Playing/Stopped", host),
        };

        let res = client
            .post(&url)
            .headers(headers)
            .json(&playing_body)
            .send();

        match res {
            Ok(res) => {
                if switch == true {
                    println!("标记播放开始，服务状态: {}", res.status());
                } else {
                    println!("标记播放结束，服务状态: {}", res.status());
                }
            }
            Err(_) => println!("标记播放失败"),
        }
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
