mod config;
mod error;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use config::{MPVClient, ReqClient};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;
use std::time::Duration;
use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Stdio},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <mpv://play/...>", args[0]);
        io::stderr().flush().unwrap();
        return;
    }

    let mpv_url = &args[1];

    // 匹配视频连接和外置字幕链接
    let (video_url, subfile_url) = match extract_urls(mpv_url) {
        Ok(urls) => urls,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    // 匹配视频链接中的参数
    let (host, item_id, api_key, media_source_id) = extract_params(&video_url);

    let mut mpv = MPVClient::new();
    // 指定终端输出播放进度，格式为HH:mm:ss.sss
    let playback_arg = "--term-status-msg=${playback-time/full}";
    // 强制立即打开播放器窗口
    let force_window = format!("--force-window=immediate");

    if !subfile_url.is_empty() {
        let subfile_arg = format!("--sub-file={}", subfile_url);

        mpv.arg(video_url)
            .arg(subfile_arg)
            .arg(playback_arg)
            .arg(force_window);
    } else {
        mpv.arg(video_url).arg(playback_arg).arg(force_window);
    }

    // 捕获 stdout 输出
    mpv.stdout(Stdio::piped());

    // 捕获 stderr 输出
    // mpv.stderr(Stdio::piped());

    // 启动子进程
    let mut child: Child = match mpv.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    // 标记播放状态
    playing_status(&host, &item_id, &api_key, &media_source_id, true);
    // 读取 stdout 输出
    let stdout = child.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);

    // 存储最后一次输出的时间戳
    let mut last_timestamp: Option<Duration> = None;

    // 处理 stdout 输出
    for line in reader.lines() {
        if let Ok(l) = line {
            // 时间戳格式为 "HH:mm:ss.sss"
            let parts: Vec<&str> = l.split(':').collect();
            if parts.len() == 3 {
                let hours = parts[0].parse::<u64>().unwrap_or(0);
                let minutes = parts[1].parse::<u64>().unwrap_or(0);
                let seconds_and_millis: Vec<&str> = parts[2].split('.').collect();
                if seconds_and_millis.len() == 2 {
                    let seconds = seconds_and_millis[0].parse::<u64>().unwrap_or(0);
                    let millis = seconds_and_millis[1].parse::<u64>().unwrap_or(0);
                    let duration = Duration::new(
                        hours * 3600 + minutes * 60 + seconds,
                        (millis * 1_000) as u32,
                    );
                    last_timestamp = Some(duration);
                }
            }
        }
    }

    // 使用时间戳更新播放进度
    if let Some(duration) = last_timestamp {
        let ticks = duration.as_nanos() / 100;
        // 标记为停止播放
        //playing_status(&host, &item_id, &api_key, &media_source_id, false);
        // 更新播放进度
        update_progress(ticks, &host, &item_id, &api_key, &media_source_id);
    } else {
        println!("获取播放时间失败");
    }
}

fn extract_urls(mpv_url: &str) -> Result<(String, String), String> {
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

fn extract_params(video_url: &str) -> (String, String, String, String) {
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

fn playing_status(host: &str, item_id: &str, api_key: &str, media_id: &str, switch: bool) {
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

fn update_progress(ticks: u128, host: &str, item_id: &str, api_key: &str, media_id: &str) {
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
