// ***
// 启动游戏主函数
// ***

use crate::utils::get_java_path::get_java_path;
use os_info;
use serde::Deserialize;
use serde_json;
use std::env::consts::OS;

use std::fmt::format;
use std::process::Command;
use crate::module::download::dwl_main::MinecraftPaths;

// 启动游戏结构体
struct StartGame {
    java_path: String,
    launch_args: Vec<String>,
}

// 启动游戏参数
struct StartGameArgs {
    java_path: String,
    version_id: String,
    java_version: String,  // 添加Java版本字段
}

// 定义JSON结构体
#[derive(Deserialize)]
struct GameConfig {
    memory: Option<String>, // 只保留内存设置
}

// 共享方法到前端
#[tauri::command]
pub async fn stg(startup_parameter: String, version_id: String, java_version: String) -> Result<String, String> {
    let start_game = StartGame::new(startup_parameter, version_id, java_version);
    match start_game.start_game() {
        Ok(output) => Ok(output),
        Err(e) => Err(format!("游戏启动失败: {}", e)),
    }
}

// 修改 get_game_jar_path 函数，使用通用方法
pub fn get_game_jar_path(version_id: &str) -> String {
    let paths = MinecraftPaths::new();
    let jar_path = paths.get_version_dir(version_id)
        .join(format!("{}.jar", version_id));
    
    paths.get_absolute_path(jar_path)
}

impl StartGame {
    pub fn new(startup_parameter: String, version_id: String, java_version: String) -> Self {
        let java_paths = get_java_path();
        let java_path = java_paths.iter()
            .find_map(|path| {
                let possible_paths = match OS {
                    "windows" => vec![
                        format!("{}\\bin\\javaw.exe", path),
                        format!("{}\\javaw.exe", path),
                        format!("{}\\javapath\\javaw.exe", path)
                    ],
                    _ => vec![
                        format!("{}/bin/java", path),
                        format!("{}/java", path)
                    ]
                };
                
                // 遍历所有可能的路径，检查Java版本
                for p in possible_paths {
                    if let Ok(version) = Self::get_java_version(&p) {
                        if version.contains(&java_version) {
                            return Some(p);
                        }
                    }
                }
                None
            })
            .unwrap_or_default();

        let launch_args = Self::load_launch_args(startup_parameter, &version_id);

        Self {
            java_path,
            launch_args,
        }
    }

    // 新增：获取Java版本的函数
    fn get_java_version(java_path: &str) -> Result<String, String> {
        let output = Command::new(java_path)
            .arg("-version")
            .output()
            .map_err(|e| e.to_string())?;

        let version_info = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(version_info)
    }

    pub fn load_launch_args(startup_parameter: String, version_id: &str) -> Vec<String> {
        let mut args = Vec::new();
        let info = os_info::get();
        let os_name = info.os_type().to_string();
        let os_version = info.version().to_string();
        
        // 获取路径管理结构体
        let paths = MinecraftPaths::new();
        // 获取日志配置文件路径
        let log4j_config_path = paths.get_absolute_path(
            paths.get_version_dir(version_id).join("client-1.12.xml")
        );
        // 获取客户端jar路径
        let game_jar_route = get_game_jar_path(version_id);
        // 获取解压的natives目录路径
        let natives_path = paths.get_absolute_path(
            paths.get_version_dir(version_id)
                .join(format!("{}-natives", version_id))
        );
        // 获取所有libraries的jar文件路径
        let mut classpath = paths.get_libraries_classpath();
        classpath.push(get_game_jar_path(version_id));
        // 将libraries_path中的路径用分隔符连接
        let libraries_path = classpath.iter()
            .map(|path| format!("\"{}\"", path))
            .collect::<Vec<String>>()
            .join(if OS == "windows" { ";" } else { ":" });

        // 分割内存参数并添加到启动参数中
        let memory_args: Vec<String> = startup_parameter
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        args.extend(memory_args);

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
            "-Dminecraft.launcher.version=0.1.1".to_string(),
            format!("-Dminecraft.client.jar={}", game_jar_route),
            format!("-Dlog4j.configurationFile={}", log4j_config_path),
            format!("-Djava.library.path={}",natives_path),
            "-cp".to_string(),
            libraries_path,
            "net.minecraft.client.main.Main".to_string(),
        ]);

        args
    }

    pub fn start_game(&self) -> Result<String, String> {
        let mut command = match OS {
            "windows" | "linux" | "macos" => Command::new(&self.java_path),
            _ => return Err("不支持的操作系统".to_string()),
        };

        command.args(&self.launch_args);

        // 打印启动命令和参数
        println!("启动Java: {}", &self.java_path);
        println!("启动参数: {:?}", &self.launch_args);

        // 执行命令并等待输出
        match command.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined_output = format!("标准输出:\n{}\n错误输出:\n{}", stdout, stderr);
                println!("游戏启动成功");
                Ok(combined_output)
            }
            Err(e) => {
                println!("游戏启动失败: {}", e);
                Err(e.to_string())
            }
        }
    }
}
