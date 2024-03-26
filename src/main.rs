#![windows_subsystem="windows"]

mod config;
mod error;

use crate::config::Config;
use std::{
    env,
    io::{self, Write},
    process::{Command, Stdio},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <vlc://http...>", args[0]);
        io::stderr().flush().unwrap();
        return;
    }

    let mpv_url = &args[1];

    let video_url = match extract_urls(mpv_url) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Error: {}", e);
            io::stderr().flush().unwrap();
            return;
        }
    };

    let mpv_command = Config::load().unwrap().mpv;
    let mut mpv = Command::new(mpv_command);

    mpv.arg(video_url);
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

fn extract_urls(mpv_url: &str) -> Result<String, String> {
    let url = mpv_url.strip_prefix("vlc://").expect("url is not correct.");

    if url.starts_with("http//") {
        let valid_url = String::from("http://") + &url[6..];
        Ok(valid_url)
    } else if url.starts_with("https//") {
        let valid_url = String::from("https://") + &url[7..];
        Ok(valid_url)
    } else {
        let valid_url = url.to_string();
        Ok(valid_url)
    }
}
