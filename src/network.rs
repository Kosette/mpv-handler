pub mod extractor {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use regex::Regex;

    pub fn extract_urls(mpv_url: &str) -> Result<(String, String), String> {
        let url = mpv_url
            .trim_end_matches('=')
            .strip_prefix("mpv://play/")
            .unwrap();

        if url.contains("/?subfile=") {
            let parts: Vec<&str> = url.splitn(2, "/?subfile=").collect();
            let video_url = URL_SAFE_NO_PAD.decode(&parts[0]).unwrap();
            let subfile_url = if parts[1].contains("&") {
                let sub_parts: Vec<&str> = parts[1].splitn(2, "&").collect();
                URL_SAFE_NO_PAD.decode(&sub_parts[0]).unwrap()
            } else {
                URL_SAFE_NO_PAD.decode(&parts[1]).unwrap()
            };

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

pub mod request {
    use std::f64;

    use super::request;
    use crate::config::Config;
    use reqwest::blocking::Client;
    use reqwest::header::{HeaderMap, HeaderValue};
    use serde_json::json;

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

        let res = request::new()
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

    pub fn start_playing(host: &str, item_id: &str, api_key: &str, media_id: &str) {
        let client = request::new();

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

        let url = format!("{}/emby/Sessions/Playing", host);

        let res = client
            .post(&url)
            .headers(headers)
            .json(&playing_body)
            .send();

        match res {
            Ok(res) => {
                println!("标记播放开始，服务状态: {}", res.status());
            }
            Err(_) => println!("标记播放失败"),
        }
    }

    pub fn get_chapter_info(host: &str, item_id: &str, api_key: &str) -> String {
        let client = request::new();

        let mut headers = HeaderMap::new();
        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key).unwrap());

        let url = format!("{}/emby/Items?Ids={}", host, item_id);

        let json: serde_json::Value = client
            .get(&url)
            .headers(headers)
            .send()
            .unwrap()
            .json()
            .expect("请求章节信息错误");

        let chapter_info = if json["Items"][0]["Type"] == "Episode" {
            let series_name = Box::new(&json["Items"][0]["SeriesName"]);
            let season = Box::new(&json["Items"][0]["ParentIndexNumber"]);
            let episode = Box::new(&json["Items"][0]["IndexNumber"]);
            let title = Box::new(&json["Items"][0]["Name"]);
            format!("{} - S{}E{} - {}", series_name, season, episode, title)
        } else if json["Items"][0]["Type"] == "Movie" {
            let title = Box::new(&json["Items"][0]["Name"]);
            format!("{}", title)
        } else {
            "".to_string()
        };
        return chapter_info;
    }

    fn get_user_id(host: &str, api_key: &str) -> String {
        let client = request::new();

        let mut headers = HeaderMap::new();
        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key).unwrap());

        let url = format!("{}/emby/Sessions", host);

        let json: serde_json::Value = client
            .get(&url)
            .headers(headers)
            .send()
            .unwrap()
            .json()
            .expect("请求用户id错误");

        let user_id = Box::new(&json[0]["UserId"]);
        format!("{}", user_id)
    }

    pub fn get_start_position(host: &str, api_key: &str, item_id: &str) -> f64 {
        let client = request::new();
        let mut user_id = get_user_id(host, api_key);
        user_id = user_id.trim_matches('"').to_string();

        let mut headers = HeaderMap::new();
        headers.insert("X-Emby-Token", HeaderValue::from_str(api_key).unwrap());

        let url = format!("{}/emby/Users/{}/Items?Ids={}", host, user_id, item_id);

        let json: serde_json::Value = client
            .get(&url)
            .headers(headers)
            .send()
            .unwrap()
            .json()
            .expect("请求播放位置信息错误");

        let percentage = json["Items"][0]["UserData"]["PlayedPercentage"].as_f64();
        if let Some(t) = percentage {
            t
        } else {
            0.0
        }
    }
}
