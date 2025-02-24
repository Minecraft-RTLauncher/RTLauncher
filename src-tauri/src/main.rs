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
