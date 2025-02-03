// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod module;
mod router;
mod utils;
mod Setting;

use api::login::get_code;
use module::download::dwl_main::get_version_manifest;
use module::download::dwl_main::dwl_version_manifest;
use utils::get_java_path::get_java_path;
use module::start_game::stg_main::stg;
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_code, get_version_manifest,dwl_version_manifest, get_java_path, stg])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
