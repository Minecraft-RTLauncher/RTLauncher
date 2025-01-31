// ***
// 获取java_home路径
// ***

use std::env::consts::OS;
use std::path::Path;
use std::process::Command;


// 主方法
#[tauri::command]
pub fn get_java_path() -> String {
    // 首先尝试从环境变量获取 JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        if Path::new(&java_home).exists() {
            return java_home;
        }
    }

    // 根据不同操作系统自动获取 Java 路径
    match OS {
        "windows" => get_windows_java_path(),
        "macos" => get_macos_java_path(),
        "linux" => get_linux_java_path(),
        _ => String::new(),
    }
}

// 获取windows的java_home路径
#[allow(dead_code)]
fn get_windows_java_path() -> String {
    // 通过 where java 命令查找 Java 路径
    if let Ok(output) = Command::new("where").arg("java").output() {
        if let Ok(path) = String::from_utf8(output.stdout) {
            if let Some(java_path) = path.lines().next() {
                // 获取 Java 可执行文件的父目录的父目录
                if let Some(parent) = Path::new(java_path).parent().and_then(|p| p.parent()) {
                    return parent.to_string_lossy().into_owned();
                }
            }
        }
    }
    String::new()
}

// 获取macos的java_home路径
#[allow(dead_code)]
fn get_macos_java_path() -> String {
    // 使用 /usr/libexec/java_home 命令获取 Java 路径
    if let Ok(output) = Command::new("/usr/libexec/java_home").output() {
        if let Ok(path) = String::from_utf8(output.stdout) {
            let path = path.trim();
            if Path::new(path).exists() {
                return path.to_string();
            }
        }
    }
    String::new()
}

// 获取linux的java_home路径
#[allow(dead_code)]
fn get_linux_java_path() -> String {
    // 1. 首先尝试使用 which java 找到 java 可执行文件
    if let Ok(output) = Command::new("which").arg("java").output() {
        if let Ok(java_path) = String::from_utf8(output.stdout) {
            let java_path = java_path.trim();
            // 如果是符号链接，尝试解析真实路径
            if let Ok(output) = Command::new("readlink").arg("-f").arg(java_path).output() {
                if let Ok(real_path) = String::from_utf8(output.stdout) {
                    let real_path = real_path.trim();
                    // 获取 Java 可执行文件的父目录的父目录
                    if let Some(parent) = Path::new(real_path).parent().and_then(|p| p.parent()) {
                        return parent.to_string_lossy().into_owned();
                    }
                }
            }
        }
    }

    // 2. 如果上述方法失败，尝试常见的安装位置
    let common_paths = ["/usr/lib/jvm", "/usr/java", "/opt/java"];

    for &base_path in &common_paths {
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().contains("java") {
                    return path.to_string_lossy().into_owned();
                }
            }
        }
    }

    String::new()
}

// 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_java_path() {
        println!("{}", get_java_path());
    }
}
