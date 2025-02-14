// ***
// 主函数
// ***

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod Setting;
mod api;
mod module;
mod router;
mod utils;

use api::login::get_code;
use module::download::dwl_main::dwl_version_manifest;
use module::download::dwl_main::get_version_manifest;
use module::start_game::stg_main::stg;
use utils::export_bat::export_bat;
use utils::get_java_path::get_java_path;
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_code,
            get_version_manifest,
            dwl_version_manifest,
            get_java_path,
            stg,
            export_bat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
