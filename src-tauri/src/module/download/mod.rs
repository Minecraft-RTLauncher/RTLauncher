pub mod dwl_main;
pub mod decompression;
pub mod paths;

use std::env::consts::OS;

// 获取用户系统
pub fn get_user_os() -> String {
    OS.to_string()
}

