#![cfg_attr(
    all(target_os = "windows", not(feature = "console")),
    windows_subsystem = "windows"
)]

mod config;
mod network;

use crate::network::{extractor, property, request};
use anyhow::{anyhow, Context, Result};
use config::MPVClient;
use extractor::M4;
use network::request::{construct_headers, get_proxy, get_ua, get_user_id, playing_status};
use std::env;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Child;
use std::result::Result::Ok;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::runtime::{self, Runtime};

fn deviceid_gen() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create runtime")
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("DEVICE_ID", deviceid_gen());

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        return Err(anyhow!("Usage: {} <mpv://play/...>", args[0]));
    }

    let mpv_url = &args[1];

    // 匹配视频连接和外置字幕链接
    let (video_url, subfile_url) = match extractor::extract_urls(mpv_url) {
        Ok(urls) => urls,
        Err(e) => {
            return Err(anyhow!("Error: {}", e));
        }
    };

    // 匹配视频链接中的参数
    let M4 {
        host,
        item_id,
        media_source_id,
        api_key,
    } = extractor::extract_params(&video_url)?;

    // 开启ipc-server
    #[cfg(windows)]
    let ipc_server = "--input-ipc-server=\\\\.\\pipe\\mpvsocket";
    #[cfg(unix)]
    let ipc_server = "--input-ipc-server=/tmp/mpvsocket";

    // 指定日志输出等级
    let msg_level = "--msg-level=all=error";

    // 强制立即打开播放器窗口
    let force_window = "--force-window=immediate";
    // set volume to 75%
    let vol_arg = "--volume=85";

    // 设置mpv请求的UA
    let ua_arg = format!("--user-agent={}", get_ua()?);

    // 设置proxy
    let proxy_arg = format!("--http-proxy={}", get_proxy()?);

    // 设置请求头
    let user_id = get_user_id(&host, &api_key).await?;
    let headers = construct_headers(&api_key, &user_id.user_id).await?;

    // 获取重定向之后的推流链接
    // let video_url = get_redirect(
    //     format!(
    //         "{}&PlaySessionId={}",
    //         video_url,
    //         user_id.unwrap().play_session_id
    //     ),
    //     headers.clone(),
    // )
    // .await;

    // 显示媒体标题信息
    let chapter_info = request::get_chapter_info(&host, &item_id, headers.clone()).await?;

    // 获取视频播放进度
    let start_ticks =
        request::get_start_position(&host, &api_key, &item_id, headers.clone()).await?;

    let start_arg = format!("--start={}", start_ticks / 10_000_000_u64);
    let title_arg = format!("--force-media-title={}", chapter_info);

    let mut mpv = MPVClient::build()?;

    if !subfile_url.is_empty() {
        let sub_arg = format!("--sub-file={}", subfile_url);
        mpv.arg(video_url)
            .arg(sub_arg)
            .arg(ua_arg)
            .arg(vol_arg)
            .arg(ipc_server)
            .arg(msg_level)
            .arg(force_window)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
        #[cfg(windows)]
        mpv.creation_flags(134_217_728u32);
    } else {
        mpv.arg(video_url)
            .arg(ua_arg)
            .arg(vol_arg)
            .arg(ipc_server)
            .arg(msg_level)
            .arg(force_window)
            .arg(title_arg)
            .arg(start_arg)
            .arg(proxy_arg);
        #[cfg(windows)]
        mpv.creation_flags(134_217_728u32);
    }

    // 启动子进程
    let mut child: Child = match mpv.spawn() {
        Ok(child) => child,
        Err(e) => {
            return Err(anyhow!("Error: {}", e));
        }
    };

    // 检测进程退出状态
    fn is_process_running(child: &mut Child) -> bool {
        std::thread::sleep(Duration::from_secs(2));

        match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    let mut ticks = start_ticks;

    // 标记播放开始
    let _ = request::playing_status(
        ticks,
        &host,
        &item_id,
        &api_key,
        &media_source_id,
        request::PlayStatus::Play,
        headers.clone(),
    )
    .await;

    let mut last_print = Instant::now();

    // 上传播放进度
    while is_process_running(&mut child) {
        if last_print.elapsed() >= Duration::from_secs(10) {
            #[cfg(windows)]
            let time_pos = property::get_time_pos_win();
            #[cfg(unix)]
            let time_pos = property::get_time_pos_unix();

            if let Ok(duration) = time_pos {
                ticks = duration.parse::<f64>().context("Failed to parse ticks")? as u64
                    * 10_000_000_u64;
                // 更新进度
                let _ = request::playing_status(
                    ticks,
                    &host,
                    &item_id,
                    &api_key,
                    &media_source_id,
                    request::PlayStatus::Progress,
                    headers.clone(),
                )
                .await;

                last_print = Instant::now();
            } else {
                println!("更新播放时间失败")
            }
        }
    }

    // 标记播放结束
    let _ = playing_status(
        ticks,
        &host,
        &item_id,
        &api_key,
        &media_source_id,
        request::PlayStatus::Stop,
        headers,
    )
    .await;

    Ok(())
}
