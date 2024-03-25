mod config;
mod error;

use config::{Config, Extractor, MPVClient, ReqClient};
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
    let (video_url, subfile_url) = match Extractor::extract_urls(mpv_url) {
        Ok(urls) => urls,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    // 匹配视频链接中的参数
    let (host, item_id, api_key, media_source_id) = Extractor::extract_params(&video_url);

    // 指定终端输出播放进度，格式为HH:mm:ss.sss
    let playback_arg = "--term-status-msg=${playback-time/full}";
    // 强制立即打开播放器窗口
    let force_window = "--force-window=immediate";
    // 显示媒体标题信息
    let chapter_info = ReqClient::get_chapter_info(&host, &item_id, &api_key);
    println!("Chapter: {}", chapter_info);
    let title_arg = format!("--force-media-title={}", chapter_info);

    let mut mpv = MPVClient::new();

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
            .arg(playback_arg)
            .arg(force_window)
            .arg(title_arg)
            .arg(proxy_arg);
    } else {
        mpv.arg(video_url)
            .arg(playback_arg)
            .arg(force_window)
            .arg(title_arg)
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
    ReqClient::playing_status(&host, &item_id, &api_key, &media_source_id, true);
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
        // ReqClient::playing_status(&host, &item_id, &api_key, &media_source_id, false);
        // 更新播放进度
        ReqClient::update_progress(ticks, &host, &item_id, &api_key, &media_source_id);
    } else {
        println!("获取播放时间失败");
    }
}
