// ***
// 导出启动脚本
// ***

use crate::module::start_game::stg_main::{StartGame, get_game_jar_path};
use crate::module::download::dwl_main::MinecraftPaths;

#[tauri::command]
pub fn export_bat(startup_parameter: String, version_id: String, java_version: String) -> String {
    // 创建StartGame实例获取启动参数
    let start_game = StartGame::new(startup_parameter, version_id, java_version);
    
    // 获取工作目录
    let paths = MinecraftPaths::new();
    let work_dir = paths.base_dir.display().to_string();
    
    // 组装批处理文件内容
    let mut bat_content = String::new();
    
    // 添加切换工作目录命令
    bat_content.push_str(&format!("cd /d \"{}\"\n", work_dir));
    
    // 添加启动命令
    bat_content.push_str(&format!("\"{}\"{}", 
        start_game.java_path,
        start_game.launch_args.iter()
            .map(|arg| format!(" \"{}\"", arg))
            .collect::<String>()
    ));
    
    // 添加暂停命令，这样在游戏关闭后窗口不会立即关闭
    bat_content.push_str("\npause");
    
    bat_content
}



