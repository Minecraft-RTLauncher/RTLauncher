// ***
// 解压主函数
// ***

use super::get_user_os;
use std::fs::File;
use zip;


// 获取系统CPU架构
fn get_cpu_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "64"
    } else if cfg!(target_arch = "x86") {
        "32"
    } else {
        "64" // 默认64位
    }
}

// 检查文件是否应该被解压
fn should_extract_file(filename: &str) -> bool {
    let os = get_user_os();
    let arch = get_cpu_arch();
    
    let filename = filename.to_lowercase();
    
    // 排除不需要的文件
    if filename.contains("meta-inf") || 
       filename.ends_with(".txt") || 
       filename.ends_with(".git") {
        return false;
    }
    
    // 根据操作系统和架构进行过滤
    match os.as_str() {
        "windows" => {
            if arch == "64" {
                !filename.contains("32.dll") && 
                (filename.ends_with(".dll") || filename.ends_with(".so") || filename.ends_with(".dylib"))
            } else {
                !filename.contains("64.dll") && 
                (filename.ends_with(".dll") || filename.ends_with(".so") || filename.ends_with(".dylib"))
            }
        },
        "osx" => filename.ends_with(".dylib"),
        "linux" => filename.ends_with(".so"),
        _ => false
    }
}

// 解压文件
pub fn decompression(path: &str, version_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 开始解压文件: {}", path);
    println!("💻 当前系统: {}, CPU架构: {}", get_user_os(), get_cpu_arch());
    
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // 构建natives目录路径
    let minecraft_dir = std::path::Path::new(".minecraft");
    let version_dir = minecraft_dir.join("version").join(version_id);
    let natives_dir = version_dir.join(format!("{}-natives", version_id));
    
    println!("📂 解压目标目录: {}", natives_dir.display());
    
    // 确保natives目录存在
    std::fs::create_dir_all(&natives_dir)?;
    
    // 解压文件
    let mut extracted_count = 0;
    let mut skipped_count = 0;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let filename = file.name().to_string();
        
        // 检查是否需要解压此文件
        if should_extract_file(&filename) {
            // 提取文件名(不包含路径)
            let simple_name = std::path::Path::new(&filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&filename);
            
            // 直接解压到natives目录根目录
            let outpath = natives_dir.join(simple_name);
            
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            
            println!("✅ 已解压: {}", simple_name);
            extracted_count += 1;
        } else {
            println!("⏭️ 已跳过: {}", filename);
            skipped_count += 1;
        }
    }
    
    println!("📊 解压完成:");
    println!("- 成功解压: {} 个文件", extracted_count);
    println!("- 已跳过: {} 个文件", skipped_count);
    
    Ok(())
}
