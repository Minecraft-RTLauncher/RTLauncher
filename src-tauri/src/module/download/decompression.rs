use super::get_user_os;
use std::fs::File;
use std::io::Read;
use zip;

// 修改解压方法，添加 version_id 参数
pub fn decompression(path: &str, version_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始解压文件: {}", path);
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // 构建natives目录路径 (.minecraft/version/版本号/版本号-natives)
    let minecraft_dir = std::path::Path::new(".minecraft");
    let version_dir = minecraft_dir.join("version").join(version_id);
    let natives_dir = version_dir.join(format!("{}-natives", version_id));
    
    println!("解压目标目录: {}", natives_dir.display());
    
    // 创建natives目录
    std::fs::create_dir_all(&natives_dir)?;
    
    // 解压文件
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = natives_dir.join(file.name());
        
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        
        println!("已解压: {}", file.name());
    }
    
    println!("解压完成: {}", path);
    Ok(())
}
