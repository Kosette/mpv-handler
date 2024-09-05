#![windows_subsystem = "windows"]

mod config;
mod error;
mod network;

use crate::network::{extractor, property, request};
use config::{Config, MPVClient};
use std::env;
use std::time::{Duration, Instant};
use std::{
    io::{self, Write},
    process::Child,
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

    // 开启ipc-server
    let ipc_server = "--input-ipc-server=\\\\.\\pipe\\mpvsocket";
    // 强制立即打开播放器窗口
    let force_window = "--force-window=immediate";
    // set volume to 75%
    let vol_arg = "--volume=85";

    // 显示媒体标题信息
    let chapter_info = request::get_chapter_info(&host, &item_id, &api_key);
    let title_arg = format!("--force-media-title={}", chapter_info);

    // 获取视频播放进度
    let start_pos = request::get_start_position(&host, &api_key, &item_id);
    let start_arg = format!("--start={}%", start_pos);

    // 设置mpv请求的UA
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
    let ua_arg = format!("--user-agent={}", ua);

    // 设置proxy
    let raw_proxy = Config::load().expect("获取自定义配置失败").proxy;
    let proxy = raw_proxy.unwrap_or("".to_string());
    let proxy_arg = format!("--http-proxy={}", proxy);

    let mut mpv = MPVClient::build();

    if !subfile_url.is_empty() {
        let sub_arg = format!("--sub-file={}", subfile_url);
        mpv.arg(video_url)
            .arg(sub_arg)
            .arg(ua_arg)
            .arg(vol_arg)
            .arg(ipc_server)
            .arg(force_window)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
    } else {
        mpv.arg(video_url)
            .arg(ua_arg)
            .arg(vol_arg)
            .arg(ipc_server)
            .arg(force_window)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
    }

    // 启动子进程
    let mut child: Child = match mpv.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    // 检测进程退出状态
    fn is_process_running(child: &mut Child) -> bool {
        match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    // 标记播放状态
    request::start_playing(&host, &item_id, &api_key, &media_source_id);

    let mut last_print = Instant::now();

    while is_process_running(&mut child) {
        let time_pos = property::get_time_pos();

        if last_print.elapsed() >= Duration::from_secs(10) {
            if let Ok(duration) = time_pos {
                let ticks: u64 = duration.parse::<f64>().unwrap() as u64 * 10_000_000_u64;
                // 更新进度
                request::update_progress(ticks, &host, &item_id, &api_key, &media_source_id);
                last_print = Instant::now();
            } else {
                println!("更新播放时间失败")
            }
        }
    }
}
