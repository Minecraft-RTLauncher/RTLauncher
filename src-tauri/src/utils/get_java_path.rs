// ***
// 获取java_home路径
// ***

use std::env::consts::OS;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;


// 主方法
#[tauri::command]
pub fn get_java_path() -> Vec<String> {
    let mut java_paths = Vec::new();

    // 首先尝试从环境变量获取 JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        if Path::new(&java_home).exists() {
            java_paths.push(java_home);
        }
    }

    // 根据不同操作系统自动获取 Java 路径
    let mut system_paths = match OS {
        "windows" => get_windows_java_path(),
        "macos" => get_macos_java_path(),
        "linux" => get_linux_java_path(),
        _ => Vec::new(),
    };

    // 合并路径并去重
    java_paths.append(&mut system_paths);
    java_paths.sort();
    java_paths.dedup();
    java_paths
}

// 获取windows的java_home路径
#[allow(dead_code)]
fn get_windows_java_path() -> Vec<String> {
    let mut paths = Vec::new();

    // 通过 where java 命令查找
    if let Ok(output) = Command::new("where").arg("java").output() {
        if let Ok(path) = String::from_utf8(output.stdout) {
            for java_path in path.lines() {
                if let Some(parent) = Path::new(java_path).parent().and_then(|p| p.parent()) {
                    paths.push(parent.to_string_lossy().into_owned());
                }
            }
        }
    }

    // 扫描文件系统查找其他安装
    let mut scan_paths = scan_for_java_installation();
    paths.append(&mut scan_paths);
    paths
}

// 获取macos的java_home路径
#[allow(dead_code)]
fn get_macos_java_path() -> Vec<String> {
    let mut paths = Vec::new();

    // 通过 /usr/libexec/java_home -V 命令获取所有版本
    if let Ok(output) = Command::new("/usr/libexec/java_home")
        .arg("-V")
        .output() {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            for line in stderr.lines() {
                if line.contains("Java SE") {
                    if let Some(path) = line.split("at ").nth(1) {
                        let path = path.trim().trim_matches('"');
                        if Path::new(path).exists() {
                            paths.push(path.to_string());
                        }
                    }
                }
            }
        }
    }

    // 扫描文件系统查找其他安装
    let mut scan_paths = scan_for_java_installation();
    paths.append(&mut scan_paths);
    paths
}

// 获取linux的java_home路径
#[allow(dead_code)]
fn get_linux_java_path() -> Vec<String> {
    let mut paths = Vec::new();

    // 通过 update-alternatives --list java 获取所有安装
    if let Ok(output) = Command::new("update-alternatives")
        .arg("--list")
        .arg("java")
        .output() {
        if let Ok(path_list) = String::from_utf8(output.stdout) {
            for java_path in path_list.lines() {
                if let Some(parent) = Path::new(java_path).parent().and_then(|p| p.parent()) {
                    paths.push(parent.to_string_lossy().into_owned());
                }
            }
        }
    }

    // 扫描文件系统查找其他安装
    let mut scan_paths = scan_for_java_installation();
    paths.append(&mut scan_paths);
    paths
}

// 通用的Java安装扫描函数
fn scan_for_java_installation() -> Vec<String> {
    let mut paths = Vec::new();
    let search_paths = match OS {
        "windows" => vec![
            "C:\\Program Files\\Java",
            "C:\\Program Files (x86)\\Java",
            "D:\\Program Files\\Java",
            "D:\\Program Files (x86)\\Java",
            "D:\\.xmcl\\jre\\java-runtime-delta",
        ],
        "macos" => vec![
            "/Library/Java/JavaVirtualMachines",
            "/System/Library/Java/JavaVirtualMachines",
            "/usr/local/opt/java",
            "/opt/homebrew/opt/java",
        ],
        "linux" => vec![
            "/usr/lib/jvm",
            "/usr/java",
            "/opt/java",
            "/usr/local/java",
        ],
        _ => vec![],
    };

    for base_path in search_paths {
        if !Path::new(base_path).exists() {
            continue;
        }

        let walker = WalkDir::new(base_path)
            .follow_links(true)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let file_name = e.file_name().to_string_lossy().to_lowercase();
                !file_name.starts_with(".") && 
                (file_name.contains("jdk") || 
                 file_name.contains("jre") || 
                 file_name.contains("java"))
            });

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let java_exec = match OS {
                    "windows" => path.join("bin").join("java.exe"),
                    _ => path.join("bin").join("java"),
                };

                if java_exec.exists() {
                    paths.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    paths
}

// 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_java_path() {
        let paths = get_java_path();
        for path in paths {
            println!("Found Java at: {}", path);
        }
    }
}
