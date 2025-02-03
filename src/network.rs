pub mod extractor {
    use anyhow::{anyhow, Context, Result};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use regex::Regex;
    use url::Url;

    pub fn extract_urls(mpv_url: &str) -> Result<(String, String)> {
        let url = mpv_url
            .trim_end_matches('=')
            .strip_prefix("mpv://play/")
            .ok_or(anyhow!("Invalid URL scheme"))?;

        if url.contains("/?subfile=") {
            let parts: Vec<&str> = url.splitn(2, "/?subfile=").collect();
            let video_url = URL_SAFE_NO_PAD
                .decode(parts[0])
                .context("Failed to decode video URL")?;
            let subfile_url = if parts[1].contains('&') {
                let sub_parts: Vec<&str> = parts[1].splitn(2, '&').collect();
                URL_SAFE_NO_PAD
                    .decode(sub_parts[0])
                    .context("Failed to decode subtitle URl")?
            } else {
                URL_SAFE_NO_PAD
                    .decode(parts[1])
                    .context("Failed to deocde subtitle URL")?
            };

            Ok((
                String::from_utf8(video_url)?,
                String::from_utf8(subfile_url)?,
            ))
        } else {
            let video_url = URL_SAFE_NO_PAD
                .decode(url)
                .context("Failed to decode video URL")?;
            Ok((String::from_utf8(video_url)?, String::new()))
        }
    }

    pub struct M4 {
        pub host: String,
        pub item_id: String,
        pub media_source_id: String,
        pub api_key: String,
    }

    pub fn extract_params(video_url: &str) -> Result<M4> {
        let url = Url::parse(video_url).context("Invalid streaming url")?;

        let host = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().ok_or(anyhow!("Hostname not found"))?
        );

        // 提取 MediaSourceId
        let Some(media_source_id) = url
            .query_pairs()
            .find(|(key, _)| key == "MediaSourceId")
            .map(|(_, value)| value)
        else {
            return Err(anyhow!("Failed to extract MediaSourceId"));
        };

        // 提取 api_key
        let Some(api_key) = url
            .query_pairs()
            .find(|(key, _)| key == "api_key")
            .map(|(_, value)| value)
        else {
            return Err(anyhow!("Failed to extract api_key"));
        };

        let pattern = Regex::new(r"^.*/videos/(\d+)/.*")?;

        // 匹配并提取 item_id
        let item_id = if let Some(captures) = pattern.captures(url.path()) {
            String::from(&captures[1])
        } else {
            return Err(anyhow!("Failed to extract ItemId"));
        };

        Ok(M4 {
            host,
            item_id,
            media_source_id: media_source_id.to_string(),
            api_key: api_key.to_string(),
        })
    }
}

pub mod request {

    use super::request;
    use crate::config::{Config, DEFAULT_UA};
    use anyhow::{anyhow, Context, Result};
    use reqwest::header::{HeaderMap, HeaderValue};
    use reqwest::Client;
    use serde_json::{json, Value};
    use std::env;
    use std::sync::OnceLock;

    // 构造请求标头
    pub async fn construct_headers(api_key: &str, user_id: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key)?);
        headers.insert(
            "X-Emby-Device-Id",
            HeaderValue::from_str(&env::var("DEVICE_ID")?.to_string())?,
        );
        headers.insert(
            "X-Emby-Device-Name",
            HeaderValue::from_str(&get_device_name())?,
        );
        headers.insert("X-Emby-Client", HeaderValue::from_static("Emby"));
        headers.insert("X-Emby-User-Id", HeaderValue::from_str(user_id)?);

        Ok(headers)
    }

    fn get_device_name() -> String {
        // 尝试在类 Unix 系统上获取主机名
        #[cfg(unix)]
        {
            env::var("HOSTNAME").unwrap_or_else(|_| "Unknown".to_string())
        }

        // 尝试在 Windows 系统上获取主机名
        // 如果是 Windows 系统，使用 "COMPUTERNAME" 环境变量
        #[cfg(windows)]
        {
            env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string())
        }
    }

    // 获取UA，默认为ExoPlayer
    pub fn get_ua() -> Result<String> {
        match Config::load().context("Failed to load config")?.useragent {
            Some(ua) => {
                if ua.is_empty() {
                    Ok(DEFAULT_UA.to_string())
                } else {
                    Ok(ua)
                }
            }
            None => Ok(DEFAULT_UA.to_string()),
        }
    }

    // 获取代理链接，默认为空
    pub fn get_proxy() -> Result<String> {
        match Config::load().context("Failed to load config")?.proxy {
            Some(proxy) => Ok(proxy),
            None => Ok("".to_string()),
        }
    }

    fn client() -> &'static reqwest::Client {
        static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
        CLIENT.get_or_init(|| request::build().expect("Failed to build Client"))
    }

    fn build() -> Result<reqwest::Client> {
        let proxy = get_proxy()?;
        let ua = get_ua()?;

        if proxy.is_empty() {
            Ok(Client::builder().user_agent(ua).build()?)
        } else {
            println!("正在使用代理访问: {}", proxy);
            let req_proxy = reqwest::Proxy::all(proxy).context("Failed to set proxy")?;

            Ok(Client::builder().proxy(req_proxy).user_agent(ua).build()?)
        }
    }

    // 获取重定向推流链接
    pub async fn _get_redirect(url: String, headers: HeaderMap) -> Result<String> {
        let proxy = get_proxy()?;

        let ua = get_ua()?;

        let client = if proxy.is_empty() {
            Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .user_agent(ua)
                .default_headers(headers)
                .build()?
        } else {
            Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .proxy(reqwest::Proxy::all(proxy).expect("设置代理失败"))
                .user_agent(ua)
                .default_headers(headers)
                .build()?
        };

        let mut current_url = url;
        let mut timer = 0_u8;

        while timer < 3 {
            let response = client.get(&current_url).send().await?;

            if response.status().is_redirection() {
                if let Some(location) = response.headers().get("location") {
                    current_url = location.to_str()?.to_string();
                } else {
                    break;
                }
            } else {
                break;
            }

            timer += 1;
        }

        Ok(current_url)
    }

    pub enum PlayStatus {
        Play,
        Progress,
        Stop,
    }

    impl std::fmt::Display for PlayStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let str = match self {
                PlayStatus::Play => "开始播放".to_string(),
                PlayStatus::Progress => "上传进度".to_string(),
                PlayStatus::Stop => "结束播放".to_string(),
            };
            write!(f, "{}", str)
        }
    }

    pub async fn playing_status(
        ticks: u64,
        host: &str,
        item_id: &str,
        api_key: &str,
        media_source_id: &str,
        status: PlayStatus,
        headers: HeaderMap,
    ) -> Result<()> {
        let params = [("reqformat", "json")];
        let body = json!({"IsMuted":false,"IsPaused":false,"RepeatMode":"RepeatNone","SubtitleOffset":0,"PlaybackRate":1,"MaxStreamingBitrate":1_000_000_000_u64,"BufferedRanges":[],"PlayMethod":"DirectStream","PlaySessionId":&get_user_id(host, api_key).await?.play_session_id,"MediaSourceId":media_source_id,"CanSeek":true,"ItemId":item_id,"PositionTicks":ticks});

        let url = match status {
            PlayStatus::Play => format!("{}/emby/Sessions/Playing", host),
            PlayStatus::Progress => format!("{}/emby/Sessions/Playing/Progress", host),
            PlayStatus::Stop => format!("{}/emby/Sessions/Playing/Stopped", host),
        };

        let res = client()
            .post(url)
            .headers(headers)
            .query(&params)
            .json(&body)
            .send()
            .await;

        match res {
            Ok(res) => {
                println!("{}，服务状态: {}", status, res.status());
            }
            Err(_) => println!("{}出错", status),
        }
        Ok(())
    }

    pub async fn get_chapter_info(host: &str, item_id: &str, headers: HeaderMap) -> Result<String> {
        let url = format!("{}/emby/Items?Ids={}", host, item_id);

        let response = client().get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Request failed, {}", response.status()));
        }

        let json: Value = response.json().await?;

        if json["Items"][0]["Type"] == "Episode" {
            let series_name = Box::new(&json["Items"][0]["SeriesName"]);
            let season = Box::new(&json["Items"][0]["ParentIndexNumber"]);
            let episode = Box::new(&json["Items"][0]["IndexNumber"]);
            let title = Box::new(&json["Items"][0]["Name"]);
            Ok(format!(
                "{} - S{}E{} - {}",
                series_name, season, episode, title
            ))
        } else if json["Items"][0]["Type"] == "Movie" {
            Ok(json["Items"][0]["Name"].to_string())
        } else {
            Ok("".to_string())
        }
    }

    pub struct Id {
        pub user_id: String,
        pub play_session_id: String,
    }

    // 获取 UserId 和 PlaySessionId
    pub async fn get_user_id(host: &str, api_key: &str) -> Result<Id> {
        let params = [
            ("X-Emby-Token", api_key),
            ("X-Emby-Device-Id", &env::var("DEVICE_ID")?.to_string()),
            ("X-Emby-Device-Name", &get_device_name()),
            ("X-Emby-Client", "Emby"),
        ];
        let url = format!("{}/emby/Sessions", host);

        let response = client().get(url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Request failed, {}", response.status()));
        }

        let json: serde_json::Value = response.json().await?;

        let user_id = Box::new(&json[0]["UserId"]);
        let play_session_id = Box::new(&json[0]["Id"]);
        Ok(Id {
            user_id: user_id.to_string().trim_matches('"').to_string(),
            play_session_id: play_session_id.to_string().trim_matches('"').to_string(),
        })
    }

    // 获取开播进度
    pub async fn get_start_position(
        host: &str,
        api_key: &str,
        item_id: &str,
        headers: HeaderMap,
    ) -> Result<u64> {
        let user_id = get_user_id(host, api_key).await?.user_id;

        let url = format!("{}/emby/Users/{}/Items?Ids={}", host, user_id, item_id);

        let response = client().get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Request failed, {}", response.status()));
        }

        let json: serde_json::Value = response.json().await?;

        let playback_ticks = json["Items"][0]["UserData"]["PlaybackPositionTicks"].as_u64();
        Ok(playback_ticks.unwrap_or(0))
    }
}

pub mod property {
    use std::io::{Read, Write};

    #[cfg(windows)]
    use serde_json::json;
    #[cfg(unix)]
    use std::os::unix::net::UnixStream;
    #[cfg(windows)]
    use std::os::windows::io::FromRawHandle;
    #[cfg(windows)]
    use windows::{core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*};

    #[cfg(windows)]
    pub fn get_time_pos_win() -> windows::core::Result<String> {
        let pipe_name = r"\\.\pipe\mpvsocket";

        let wide_pipe_name: Vec<u16> = pipe_name.encode_utf16().chain(std::iter::once(0)).collect();

        let handle = unsafe {
            CreateFileW(
                PCWSTR::from_raw(wide_pipe_name.as_ptr()),
                (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL, // 改为普通文件属性
                Some(HANDLE::default()),
            )
        };

        if handle.is_err() {
            println!("Failed to open pipe: {:?}", handle.as_ref().err());
            return Err(handle.err().unwrap());
        }

        let handle = handle.unwrap();
        let mut file = unsafe { std::fs::File::from_raw_handle(handle.0 as *mut _) };

        let message = json!({
            "command": ["get_property", "time-pos"]
        });

        // 添加换行符
        let message_str = message.to_string() + "\n";
        // println!("Sending message: {}", message_str);
        file.write_all(message_str.as_bytes()).map_err(|e| {
            println!("Failed to write: {:?}", e);
            Error::from_win32()
        })?;

        let mut response = String::new();
        let mut buffer = [0; 1024];
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break, // 读取结束
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if response.ends_with('\n') {
                        break; // 读取到换行符，认为响应结束
                    }
                }
                Err(e) => {
                    println!("Failed to read: {:?}", e);
                    return Err(Error::from_win32());
                }
            }
        }

        let time_pos: serde_json::Value =
            serde_json::from_str(response.trim()).expect("No valid time-pos found.");
        // println!("Received response: {}", time_pos["data"]);
        let time_data = time_pos["data"].to_string();

        Ok(time_data)
    }

    #[cfg(unix)]
    use anyhow::{Context, Result};
    #[cfg(unix)]
    pub fn get_time_pos_unix() -> Result<String> {
        // 连接到 MPV 的 IPC socket
        let mut stream = UnixStream::connect("/tmp/mpvsocket")?;

        // 构造要发送的命令
        let command = r#"{ "command": ["get_property", "time-pos"] }"#;

        // 发送命令
        stream.write_all(command.as_bytes())?;
        stream.write_all(b"\n")?; // 确保以换行符结束

        // 读取响应
        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        // println!("Received response: {}", response);
        let time_pos: serde_json::Value =
            serde_json::from_str(response.trim()).context("No valid time-pos found.")?;
        // println!("Received response: {}", time_pos["data"]);
        let time_data = time_pos["data"].to_string();

        Ok(time_data)
    }
}
