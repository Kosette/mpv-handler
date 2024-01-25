mod config;
mod error;

use crate::config::Config;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use std::{
    env,
    io::{self, Write},
    process::{Command, Stdio},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <mpv://play/...>", args[0]);
        io::stderr().flush().unwrap();
        return;
    }

    let mpv_url = &args[1];

    let (video_url, subfile_url) = match extract_urls(mpv_url) {
        Ok(urls) => urls,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    let mpv_command = Config::load().unwrap().mpv;
    let mut mpv = Command::new(mpv_command);

    if !subfile_url.is_empty() {
        let subfile_arg = format!("--sub-file={}", subfile_url);
        mpv.arg(video_url).arg(subfile_arg);
    } else {
        mpv.arg(video_url);
    }

    mpv.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    match mpv.spawn() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
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
