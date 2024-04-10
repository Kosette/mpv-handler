#![windows_subsystem = "windows"]

mod config;
mod error;
mod network;

use crate::network::{extractor, request};
use config::{Config, MPVClient};
use std::env;
use std::time::Duration;
use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Stdio},
};

fn deviceid_gen() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn main() {
    env::set_var("DEVICE_ID", deviceid_gen());

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <mpv://play/...>", args[0]);
        io::stderr().flush().unwrap();
        return;
    }

    let mpv_url = &args[1];

    // 匹配视频连接和外置字幕链接
    let (video_url, subfile_url) = match extractor::extract_urls(mpv_url) {
        Ok(urls) => urls,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    // 匹配视频链接中的参数
    let (host, item_id, api_key, media_source_id) = extractor::extract_params(&video_url);

    // 指定输出等级为status
    let msg_level = "--msg-level=all=status";
    // 指定终端输出播放进度，格式为HH:mm:ss.sss
    let playback_arg = "--term-status-msg=${playback-time/full}";
    // 强制立即打开播放器窗口
    let force_window = "--force-window=immediate";
    // set volume to 75%
    let vol_arg = "--volume=85";
    // 显示媒体标题信息
    let chapter_info = request::get_chapter_info(&host, &item_id, &api_key);
    //println!("Chapter: {}", chapter_info);
    let title_arg = format!("--force-media-title={}", chapter_info);
    // 获取视频播放进度
    let start_pos = request::get_start_position(&host, &api_key, &item_id);
    let start_arg = format!("--start={}%", start_pos);

    let mut mpv = MPVClient::build();

    let raw_proxy = Config::load().expect("获取自定义配置失败").proxy;
    let proxy = match raw_proxy {
        Some(proxy) => proxy,
        None => "".to_string(),
    };
    let proxy_arg = format!("--http-proxy={}", proxy);

    if !subfile_url.is_empty() {
        let sub_arg = format!("--sub-file={}", subfile_url);
        mpv.arg(video_url)
            .arg(sub_arg)
            .arg(vol_arg)
            .arg(msg_level)
            .arg(force_window)
            .arg(playback_arg)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
    } else {
        mpv.arg(video_url)
            .arg(vol_arg)
            .arg(msg_level)
            .arg(force_window)
            .arg(playback_arg)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
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
    request::start_playing(&host, &item_id, &api_key, &media_source_id);
    // 读取 stdout 输出
    let stdout = child.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);

    // 存储最后一次输出的时间戳
    let mut last_timestamp: Option<Duration> = None;

    // 处理 stdout 输出
    for line in reader.lines().map_while(Result::ok) {
        // 时间戳格式为 "HH:mm:ss.sss"
        let parts: Vec<&str> = line.split(':').collect();
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

    // 使用时间戳更新播放进度
    if let Some(duration) = last_timestamp {
        let ticks = duration.as_secs() * 10000000;
        // 更新播放进度
        request::update_progress(ticks, &host, &item_id, &api_key, &media_source_id);
    } else {
        println!("获取播放时间失败");
    }
}
