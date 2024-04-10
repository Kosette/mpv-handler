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
            let video_url = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
            let subfile_url = if parts[1].contains('&') {
                let sub_parts: Vec<&str> = parts[1].splitn(2, '&').collect();
                URL_SAFE_NO_PAD.decode(sub_parts[0]).unwrap()
            } else {
                URL_SAFE_NO_PAD.decode(parts[1]).unwrap()
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

            (
                String::from(host),
                String::from(item_id),
                String::from(api_key),
                String::from(media_source_id),
            )
        } else {
            (String::new(), String::new(), String::new(), String::new())
        }
    }
}

pub mod request {

    use super::request;
    use crate::config::Config;
    use reqwest::blocking::Client;
    use serde_json::json;
    use std::env;
    use std::sync::OnceLock;

    fn client() -> &'static reqwest::blocking::Client {
        static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
        CLIENT.get_or_init(request::build)
    }

    pub fn build() -> reqwest::blocking::Client {
        let raw_proxy = Config::load().expect("获取自定义设置失败").proxy;
        let proxy = match raw_proxy {
            Some(proxy) => proxy,
            None => "".to_string(),
        };

        if proxy.is_empty() {
            Client::new()
        } else {
            println!("正在使用代理访问: {}", proxy);
            let req_proxy = reqwest::Proxy::all(proxy).expect("设置代理失败");
            Client::builder().proxy(req_proxy).build().unwrap()
        }
    }

    pub fn update_progress(ticks: u64, host: &str, item_id: &str, api_key: &str, media_id: &str) {
        let params = [
            ("X-Emby-Token", api_key),
            (
                "X-Emby-Device-Id",
                &env::var("DEVICE_ID").unwrap().to_string(),
            ),
            ("X-Emby-Device-Name", "Google Chrome"),
        ];
        let stopped_position =
            json!({"ItemId":item_id,"MediaSourceId":media_id,"PositionTicks":ticks});

        let res = client()
            .post(format!("{}/emby/Sessions/Playing/Stopped", host))
            // .headers(headers)
            .query(&params)
            .json(&stopped_position)
            .send();

        match res {
            Ok(res) => {
                println!("正在回传进度，请求状态: {}", res.status());
            }
            Err(_) => println!("回传进度失败"),
        }
    }

    pub fn start_playing(host: &str, item_id: &str, api_key: &str, media_id: &str) {
        let params = [
            ("X-Emby-Token", api_key),
            (
                "X-Emby-Device-Id",
                &env::var("DEVICE_ID").unwrap().to_string(),
            ),
            ("X-Emby-Device-Name", "Google Chrome"),
        ];
        let playing_body = json!({"ItemId":item_id,"MediaSourceId":media_id});

        let url = format!("{}/emby/Sessions/Playing", host);

        let res = client().post(url).query(&params).json(&playing_body).send();

        match res {
            Ok(res) => {
                println!("标记播放开始，服务状态: {}", res.status());
            }
            Err(_) => println!("标记播放失败"),
        }
    }

    pub fn get_chapter_info(host: &str, item_id: &str, api_key: &str) -> String {
        let params = [
            ("X-Emby-Token", api_key),
            (
                "X-Emby-Device-Id",
                &env::var("DEVICE_ID").unwrap().to_string(),
            ),
            ("X-Emby-Device-Name", "Google Chrome"),
        ];
        let url = format!("{}/emby/Items?Ids={}", host, item_id);

        let json: serde_json::Value = client()
            .get(url)
            .query(&params)
            .send()
            .unwrap()
            .json()
            .expect("请求章节信息错误");

        if json["Items"][0]["Type"] == "Episode" {
            let series_name = Box::new(&json["Items"][0]["SeriesName"]);
            let season = Box::new(&json["Items"][0]["ParentIndexNumber"]);
            let episode = Box::new(&json["Items"][0]["IndexNumber"]);
            let title = Box::new(&json["Items"][0]["Name"]);
            format!("{} - S{}E{} - {}", series_name, season, episode, title)
        } else if json["Items"][0]["Type"] == "Movie" {
            json["Items"][0]["Name"].to_string()
        } else {
            "".to_string()
        }
    }

    fn get_user_id(host: &str, api_key: &str) -> String {
        let params = [
            ("X-Emby-Token", api_key),
            (
                "X-Emby-Device-Id",
                &env::var("DEVICE_ID").unwrap().to_string(),
            ),
            ("X-Emby-Device-Name", "Google Chrome"),
        ];
        let url = format!("{}/emby/Sessions", host);

        let json: serde_json::Value = client()
            .get(url)
            .query(&params)
            .send()
            .unwrap()
            .json()
            .expect("请求用户id错误");

        let user_id = Box::new(&json[0]["UserId"]);
        format!("{}", user_id)
    }

    pub fn get_start_position(host: &str, api_key: &str, item_id: &str) -> f64 {
        let mut user_id = get_user_id(host, api_key);
        user_id = user_id.trim_matches('"').to_string();

        let params = [
            ("X-Emby-Token", api_key),
            (
                "X-Emby-Device-Id",
                &env::var("DEVICE_ID").unwrap().to_string(),
            ),
            ("X-Emby-Device-Name", "Google Chrome"),
        ];
        let url = format!("{}/emby/Users/{}/Items?Ids={}", host, user_id, item_id);

        let json: serde_json::Value = client()
            .get(url)
            .query(&params)
            .send()
            .unwrap()
            .json()
            .expect("请求播放位置信息错误");

        let percentage = json["Items"][0]["UserData"]["PlayedPercentage"].as_f64();
        if let Some(t) = percentage {
            t
        } else {
            0.
        }
    }
}
