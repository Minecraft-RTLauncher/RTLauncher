use crate::utils::get_java_path::get_java_path;
use os_info;
use serde::Deserialize;
use serde_json;
use std::env::consts::OS;
use std::process::Command;

// 启动游戏结构体
struct StartGame {
    java_path: String,
    launch_args: Vec<String>,
}

// 启动游戏参数
struct StartGameArgs {
    java_path: String,
}

// 定义JSON结构体
#[derive(Deserialize)]
struct GameConfig {
    memory: Option<String>, // 只保留内存设置
}

// 共享方法到前端
#[tauri::command]
pub async fn stg(startup_parameter: String) -> Result<(), String> {
    let start_game = StartGame::new(startup_parameter);
    match start_game.start_game() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("游戏启动失败: {}", e)),
    }
}

impl StartGame {
    pub fn new(startup_parameter: String) -> Self {
        let mut java_path = get_java_path();
        // 根据操作系统添加可执行文件名
        java_path = match OS {
            "windows" => format!("{}\\bin\\java.exe", java_path),
            _ => format!("{}\\bin\\java", java_path), // Linux 和 macOS
        };

        let launch_args = Self::load_launch_args(startup_parameter);

        Self {
            java_path,
            launch_args,
        }
    }

    // 简化启动参数加载函数
    pub fn load_launch_args(startup_parameter: String) -> Vec<String> {
        let mut args = Vec::new();
        let info = os_info::get();
        let os_name = info.os_type().to_string();
        let os_version = info.version().to_string();

        // 检查是否为32位Windows系统
        let is_windows_32bit = OS == "windows" && cfg!(target_pointer_width = "32");
        if is_windows_32bit {
            args.push("-Xss1M".to_string());
        }

        // Windows系统特定参数
        if OS == "windows" {
            args.push("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".to_string());
        }

        // jvm参数
        args.extend(vec![
            "-XX:+UseG1GC".to_string(),
            "-XX:-UseAdaptiveSizePolicy".to_string(),
            "-XX:-OmitStackTraceInFastThrow".to_string(),
            format!("-Dos.name={}", os_name),
            format!("-Dos.version={}", os_version),

            "-Dminecraft.launcher.brand=RTLauncher".to_string(),
            "-Dminecraft.launcher.version=0.1.1".to_string()
        ]);

        args
    }

    pub fn start_game(&self) -> Result<(), String> {
        let mut command = match OS {
            "windows" | "linux" | "macos" => Command::new(&self.java_path),
            _ => return Err("不支持的操作系统".to_string()),
        };

        // 添加动态加载的启动参数
        command.args(&self.launch_args);

        // 打印启动命令和参数
        println!("启动命令: {}", &self.java_path);
        println!("启动参数: {:?}", &self.launch_args);

        // 执行命令
        match command.spawn() {
            Ok(_) => {
                println!("游戏启动成功");
                Ok(())
            }
            Err(e) => {
                println!("游戏启动失败: {}", e);
                Err(e.to_string())
            }
        }
    }
}
