#![windows_subsystem = "windows"]

mod config;

use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use std::result::Result::Ok;
use std::{
    env,
    process::{Command, Stdio},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid arguments.\nUsage: {} <vlc://http...>",
            args[0]
        ));
    }

    let mpv_url = &args[1];

    let video_url = match extract_urls(mpv_url) {
        Ok(url) => url,
        Err(e) => {
            return Err(anyhow!("Error: {}", e));
        }
    };

    let mpv_command = Config::load()?.mpv;
    let mut mpv = Command::new(mpv_command);

    mpv.arg(video_url);
    mpv.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    match mpv.spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error: {}", e)),
    }
}

fn extract_urls(mpv_url: &str) -> Result<String> {
    let url = mpv_url
        .strip_prefix("vlc://")
        .context("Invalid URL, should start with vlc://")?;

    if let Some(stripped) = url.strip_prefix("http//") {
        let valid_url = String::from("http://") + stripped;
        Ok(valid_url)
    } else if let Some(stripped) = url.strip_prefix("https//") {
        let valid_url = String::from("https://") + stripped;
        Ok(valid_url)
    } else {
        let valid_url = url.to_string();
        Ok(valid_url)
    }
}
