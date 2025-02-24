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
// 导出启动脚本的功能，生成一个 .bat 文件供启动游戏使用
// ***

use crate::module::start_game::stg_main::StartGame;
use std::fs::File;
use std::io::Write;

#[tauri::command]
pub async fn export_bat(
    startup_parameter: String,
    version_id: String,
    java_version: String,
    output_path: String,
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
    let full_command = format!(
        "\"{}\" {}",
        start_game.java_path,
        start_game.launch_args.join(" ")
    );

    // 生成 .bat 文件内容
    let content = format!("@echo off\r\n{}\r\npause\r\n", full_command);

    // 将内容写入指定的 .bat 文件
    match File::create(&output_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                return Err(format!("写入文件失败: {}", e));
            }
        }
        Err(e) => return Err(format!("创建文件失败: {}", e)),
    }

    Ok("批处理文件已成功导出".to_string())
}
