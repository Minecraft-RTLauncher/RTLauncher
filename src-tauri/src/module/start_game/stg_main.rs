/*
RTLauncher, a third-party Minecraft launcher built with the newest
technology and provides innovative funtionalities
Copyright (C) 2025 lutouna

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// ***
// 启动游戏主函数
// ***

use crate::utils::get_java_path::get_java_path;
use os_info;
use std::env::consts::OS;

use crate::module::download::dwl_main::MinecraftPaths;
use std::process::Command;

// 启动游戏结构体
pub struct StartGame {
    pub java_path: String,
    pub launch_args: Vec<String>,
}

// 共享方法到前端
#[tauri::command]
pub async fn stg(
    startup_parameter: String,
    version_id: String,
    java_version: String,
    asset_index_id: String,
    username: String,
) -> Result<String, String> {
    let start_game = StartGame::new(
        startup_parameter,
        version_id,
        java_version,
        asset_index_id,
        username,
    );
    match start_game.start_game() {
        Ok(output) => Ok(output),
        Err(e) => Err(format!("游戏启动失败: {}", e)),
    }
}

// 获取游戏jar路径
pub fn get_game_jar_path(version_id: &str) -> String {
    let paths = MinecraftPaths::new();
    let jar_path = paths
        .get_version_dir(version_id)
        .join(format!("{}.jar", version_id));

    paths.get_absolute_path(jar_path)
}

impl StartGame {
    pub fn new(
        startup_parameter: String,
        version_id: String,
        java_version: String,
        asset_index_id: String,
        username: String,
    ) -> Self {
        let java_paths = get_java_path();
        let java_path = java_paths
            .iter()
            .find_map(|path| {
                let possible_paths = match OS {
                    "windows" => vec![
                        format!("{}\\bin\\java.exe", path),
                        format!("{}\\java.exe", path),
                        format!("{}\\javapath\\java.exe", path),
                    ],
                    _ => vec![format!("{}/bin/java", path), format!("{}/java", path)],
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

        let launch_args =
            Self::load_launch_args(startup_parameter, &version_id, &asset_index_id, username);

        Self {
            java_path,
            launch_args,
        }
    }

    // 获取java版本
    fn get_java_version(java_path: &str) -> Result<String, String> {
        let output = Command::new(java_path)
            .arg("-version")
            .output()
            .map_err(|e| e.to_string())?;

        let version_info = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(version_info)
    }

    pub fn load_launch_args(
        startup_parameter: String,
        version_id: &str,
        asset_index_id: &str,
        username: String,
    ) -> Vec<String> {
        let mut args = Vec::new();
        let info = os_info::get();
        let os_name = info.os_type().to_string();
        let os_version = info.version().to_string();

        // 获取路径管理结构体
        let paths = MinecraftPaths::new();
        // 获取日志配置文件路径
        let log4j_config_path =
            paths.get_absolute_path(paths.get_version_dir(version_id).join("client-1.12.xml"));
        // 获取客户端jar路径
        let game_jar_route = get_game_jar_path(version_id);
        // 获取解压的natives目录路径
        let natives_path = paths.get_absolute_path(
            paths
                .get_version_dir(version_id)
                .join(format!("{}-natives", version_id)),
        );
        // 获取所有libraries的jar文件路径
        let mut classpath = paths.get_libraries_classpath();
        classpath.push(get_game_jar_path(version_id));

        // 获取classpath路径
        let libraries_path = if OS == "windows" {
            format!("{}", classpath.join(";"))
        } else {
            format!("{}", classpath.join(":"))
        };

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
            "-Dminecraft.launcher.brand=RTL".to_string(),
            "-Dminecraft.launcher.version=0.1.1".to_string(),
            format!("-Dminecraft.client.jar={}", game_jar_route),
            format!("-Dlog4j.configurationFile={}", log4j_config_path),
            format!("-Djava.library.path={}", natives_path),
            "-cp".to_string(),
            libraries_path,
            "net.minecraft.client.main.Main".to_string(),
            "--version".to_string(),
            version_id.to_string(),
            "--username".to_string(),
            username,
            "--accessToken".to_string(),
            "00000FFFFFFFFFFFFFFFFFFFFFF9E747".to_string(),
            "--gameDir".to_string(),
            paths.base_dir.to_string_lossy().into_owned(),
            "--assetsDir".to_string(),
            paths.assets_dir.to_string_lossy().into_owned(),
            "--assetIndex".to_string(),
            asset_index_id.to_string(),
        ]);

        args
    }

    pub fn start_game(&self) -> Result<String, String> {
        let mut command = match OS {
            "windows" | "linux" | "macos" => Command::new(&self.java_path),
            _ => return Err("不支持的操作系统".to_string()),
        };

        // 设置工作目录为 .minecraft 文件夹
        let paths = MinecraftPaths::new();
        command.current_dir(&paths.base_dir);

        command.args(&self.launch_args);

        // 完整的启动命令
        let full_command = format!("\"{}\" {}", &self.java_path, &self.launch_args.join(" "));
        println!("完整启动命令: {}", full_command);
        println!("工作目录: {}", paths.base_dir.display());

        // 打印启动命令和参数
        println!("启动Java: {}", &self.java_path);
        println!("启动参数: {:?}", &self.launch_args);

        // 启动游戏进程
        match command.spawn() {
            Ok(mut child) => {
                println!("游戏启动成功，进程ID: {:?}", child.id());

                // 等待游戏进程结束
                match child.wait() {
                    Ok(status) => {
                        println!("游戏进程已结束，退出状态: {}", status);
                        Ok("游戏已关闭".to_string())
                    }
                    Err(e) => {
                        println!("等待游戏进程时出错: {}", e);
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                println!("游戏启动失败: {}", e);
                Err(e.to_string())
            }
        }
    }
}
