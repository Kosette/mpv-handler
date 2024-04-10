use serde::Serialize;
use std::env;
use std::fs;
use std::io::{stdin, stdout, Read, Write};
use std::os::windows::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Output};

enum Operation {
    Install,
    Uninstall,
}

fn main() {
    if !is_vista_or_later() {
        println!("This program only works on Windows Vista and later.\n脚本支持Windows Vista及更新的系统。");
        wait_for_key_press();
        return;
    }

    if !is_admin() {
        println!("This program requires administrator privileges. Right-click to run as Administrator.\n程序需要使用管理员权限运行。右键使用管理员运行。");
        wait_for_key_press();
        return;
    }

    let args: Vec<String> = env::args().collect();

    let operation = if args.iter().any(|arg| arg == "/i") {
        Operation::Install
    } else if args.iter().any(|arg| arg == "/r") {
        Operation::Uninstall
    } else {
        choose_operation()
    };

    match operation {
        Operation::Install => {
            if !check_binary() {
                wait_for_key_press();
                return;
            }
            add_reg();
            println!("Install reg successfully!  安装注册表成功！");
            add_path();
            wait_for_key_press();
        }
        Operation::Uninstall => {
            del_reg();
            println!("Uninstall reg successfully!  卸载注册表成功！");
            wait_for_key_press();
        }
    }
}

#[derive(Debug, Serialize, Default)]
struct Config {
    mpv: String,
    proxy: String,
}

fn add_path() {
    println!("Configuring mpv path....\n正在配置mpv路径……");
    let current_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("mpv.exe");
    let parent_dir = match env::current_exe().unwrap().parent().unwrap().parent() {
        Some(path) => path.to_path_buf(),
        None => env::current_exe().unwrap().parent().unwrap().to_path_buf(),
    };

    let mpv_path = if current_path.exists() {
        println!("find mpv: \n找到mpv程序: \n{}", current_path.display());
        current_path.display().to_string()
    } else if parent_dir.join("mpv.exe").exists() {
        println!(
            "find mpv: \n找到mpv程序: \n{}",
            parent_dir.join("mpv.exe").display()
        );
        parent_dir.join("mpv.exe").display().to_string()
    } else {
        println!("mpv.exe is not found in current or parent directory.\n在当前文件夹和上层文件夹中没有发现mpv.exe.");
        println!("fill in config.toml on your own.\n请自行在config.toml中填写");
        "".to_string()
    };

    let config_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");

    let config = Config {
        mpv: mpv_path,
        ..Default::default()
    };
    let toml = toml::to_string(&config).unwrap();
    let status = fs::write(config_path, toml);

    match status {
        Ok(_) => println!("Configure complete.\n配置文件写入完成。"),
        Err(e) => println!("Configure failed.\n配置文件写入失败。\nError occur: {}", e),
    }
}

fn choose_operation() -> Operation {
    println!("Enter 'r' to uninstall registry, 'i' to install.\n输入'r'移除注册表，输入'i'安装注册表。\n( r / i )?");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "i" => Operation::Install,
        "r" => Operation::Uninstall,
        _ => choose_operation(),
    }
}

fn wait_for_key_press() {
    println!("Press Enter to continue...\n按回车键继续...");
    let mut buffer = [0u8; 1];
    stdin().read_exact(&mut buffer).unwrap();
}

fn is_vista_or_later() -> bool {
    let output = Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .unwrap_or_else(|_| Output {
            status: ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    output.status.success()
}

fn is_admin() -> bool {
    let output = Command::new("cmd")
        .args(["/C", "openfiles"])
        .output()
        .unwrap_or_else(|_| Output {
            status: ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    output.status.success()
}

fn check_binary() -> bool {
    let mpv_handler_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("mpv-handler.exe");
    let mpv_handler_conf = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");

    if !mpv_handler_path.exists() {
        println!("Please put mpv-handler.exe with cfg_tool.exe.\n请把mpv-handler.exe和cfg_tool放在同一个目录。\n");
        return false;
    }

    if !mpv_handler_conf.exists() {
        println!("If MPV not in PATH, put your config.toml here too.\n如果你的MPV播放器没有加入系统PATH，把你修改后的config.toml也放在这里。\n");
    }

    true
}

fn add_reg() {
    reg_command(&["add", "HKCR\\mpv", "/d", "URL:mpv", "/f"]);
    reg_command(&["add", "HKCR\\mpv", "/v", "URL Protocol", "/f"]);
    let handler_path = format!(
        "{}",
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("mpv-handler.exe")
            .display()
    );
    let handler_str = format!("\"{}\" \"%1\"", handler_path);
    reg_command(&[
        "add",
        "HKCR\\mpv\\shell\\open\\command",
        "/d",
        &handler_str[..],
        "/f",
    ]);
}

fn del_reg() {
    reg_command(&["delete", "HKCR\\mpv", "/f"]);
}

fn reg_command(args: &[&str]) {
    let status: ExitStatus = Command::new("reg")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !status.success() {
        println!("Error: {}", status);
    }
}
