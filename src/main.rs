use std::env;
use std::process::Command;

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查参数数量是否正确
    if args.len() != 2 {
        println!("Usage: {} <mpv-url>", args[0]);
        return;
    }

    // 解析mpv-url
    let mpv_url = &args[1];
    let raw_url = match mpv_url.strip_prefix("mpv://") {
        Some(url) => url,
        None => {
            println!("Invalid mpv-url: {}", mpv_url);
            return;
        }
    };

    // 构建mpv命令
    let mut mpv_command = Command::new("d:\\Scoop\\apps\\mpv\\current\\mpv.exe");
    mpv_command.arg(raw_url);

    // 执行mpv命令
    match mpv_command.spawn() {
        Ok(_) => println!("Playing video: {}", raw_url),
        Err(e) => println!("Error executing mpv: {}", e),
    }
}
