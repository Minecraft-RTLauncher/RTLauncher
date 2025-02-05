// ***
// 下载主方法
// ***

use crate::utils::request;
use futures::stream::{self, StreamExt};
use reqwest;
use sha1::Digest;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use super::get_user_os;
use super::decompression::decompression;
use walkdir;

pub struct Download {
    pub version_manifest_url: String, // 获取版本url
    pub id: String,
    pub version_type: String,
}

pub struct DownloadOptions {
    pub url: String,        // 下载路径
    pub version_id: String, // 版本号
}

// 修改 DownloadInfo 结构体，添加下载进度跟踪
#[derive(Clone)]
struct DownloadInfo {
    url: String,
    path: std::path::PathBuf,
    size: u64,
    downloaded: Arc<AtomicUsize>,
}

// 添加新的结构体来跟踪下载进度
#[derive(Clone)]
struct DownloadProgress {
    total: Arc<AtomicUsize>,
    current: Arc<AtomicUsize>,
    success: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
}

impl DownloadProgress {
    fn new(total: usize) -> Self {
        Self {
            total: Arc::new(AtomicUsize::new(total)),
            current: Arc::new(AtomicUsize::new(0)),
            success: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn update_success(&self) {
        self.success.fetch_add(1, Ordering::SeqCst);
    }

    fn update_failed(&self) {
        self.failed.fetch_add(1, Ordering::SeqCst);
    }

    fn get_current(&self) -> usize {
        self.success.load(Ordering::SeqCst) + self.failed.load(Ordering::SeqCst)
    }
}

// 添加新的路径管理结构体和实现
pub struct MinecraftPaths {
    pub base_dir: std::path::PathBuf,
    pub versions_dir: std::path::PathBuf,
    pub libraries_dir: std::path::PathBuf,
    pub assets_dir: std::path::PathBuf,
}

impl MinecraftPaths {
    pub fn new() -> Self {
        // 游戏文件保存路径
        let base_dir = std::path::PathBuf::from("D:\\Desktop\\.minecraft");
        Self {
            versions_dir: base_dir.join("version"),
            libraries_dir: base_dir.join("libraries"),
            assets_dir: base_dir.join("assets"),
            base_dir,
        }
    }

    pub fn get_version_dir(&self, version_id: &str) -> std::path::PathBuf {
        self.versions_dir.join(version_id)
    }

    pub fn get_natives_dir(&self, version_id: &str) -> std::path::PathBuf {
        self.get_version_dir(version_id).join(format!("{}-natives", version_id))
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.versions_dir)?;
        std::fs::create_dir_all(&self.libraries_dir)?;
        std::fs::create_dir_all(&self.assets_dir)?;
        Ok(())
    }

    // 获取绝对路径-公共方法
    pub fn get_absolute_path(&self, path: std::path::PathBuf) -> String {
        path.canonicalize()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .trim_start_matches(r"\\?\")

            .to_string()
    }

    // 获取libraries目录下所有jar文件的路径
    pub fn get_libraries_classpath(&self) -> Vec<String> {
        walkdir::WalkDir::new(&self.libraries_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .map_or(false, |ext| ext == "jar")
            })
            .map(|entry| self.get_absolute_path(entry.path().to_path_buf()))
            .collect()
    }
}

#[tauri::command]
pub async fn get_version_manifest() -> Result<serde_json::Value, String> {
    let download = Download::new(String::from(
        "https://piston-meta.mojang.com/mc/game/version_manifest.json",
    ));
    download
        .dwl_version_manifest()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dwl_version_manifest(url: String) -> Result<serde_json::Value, String> {
    let url = if url.starts_with('{') {
        // 如果输入是 JSON 字符串，尝试解析
        let parsed_json: serde_json::Value =
            serde_json::from_str(&url).map_err(|e| format!("JSON解析错误: {}", e))?;

        // 从 JSON 对象中提取 url 字段
        parsed_json
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or("JSON中未找到有效的url字段")?
            .to_string()
    } else {
        // 如果输入是普通URL字符串，直接使用
        url
    };

    let download = DownloadOptions::new(url);
    let res = download
        .dwl_version_manifest()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

impl Download {
    pub fn new(version_manifest_url: String) -> Self {
        Self {
            version_manifest_url: String::from(
                "https://piston-meta.mojang.com/mc/game/version_manifest.json",
            ),
            id: String::from(""),
            version_type: String::from(""),
        }
    }

    async fn dwl_version_manifest(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let request = request::Request::new(self.version_manifest_url.clone());
        let res = request.fetch_get().await.unwrap();
        let json_value = serde_json::from_str::<serde_json::Value>(&res)?;
        Ok(json_value)
    }
}

impl DownloadOptions {
    pub fn new(url: String) -> Self {
        Self {
            url,
            version_id: String::new(),
        }
    }

    // 下载游戏资源
    pub async fn dwl_version_manifest(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let response = request::Request::new(self.url.clone());
        let res = response.fetch_get().await?;
        let mut timings = Vec::new();

        // 解析json
        let json_value: serde_json::Value = serde_json::from_str(&res)?;
        let version_id = json_value["id"].as_str().unwrap_or("unknown");

        // 使用新的路径管理结构体
        let paths = MinecraftPaths::new();
        paths.ensure_dirs()?;

        let version_path = paths.get_version_dir(version_id);
        std::fs::create_dir_all(&version_path)?;

        let mut success_count = 0;
        let mut failed_count = 0;
        let current_os = get_user_os(); // 获取当前操作系统

        // 1. 客户端jar
        let jar_start = std::time::Instant::now();
        if let Some(client) = json_value
            .get("downloads")
            .and_then(|downloads| downloads.get("client"))
        {
            let client_url = client["url"].as_str().unwrap_or_default().to_string();
            let client_sha1 = client["sha1"].as_str().unwrap_or_default().to_string(); // 获取SHA1值
            let jar_path = version_path.join(format!("{}.jar", version_id));

            match download_and_verify_file(client_url, jar_path, &client_sha1, None, 3).await {
                Ok(info) => {
                    let duration = jar_start.elapsed();
                    timings.push(("客户端jar".to_string(), duration));
                    println!(
                        "✅ 下载成功: {} -> {} (耗时: {:.2}秒)",
                        info.url,
                        info.path.display(),
                        duration.as_secs_f64()
                    );
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ 下载失败: {}", e);
                    failed_count += 1;
                }
            }
        }

        // 2. 下载日志配置XML文件
        if let Some(logging) = json_value.get("logging") {
            if let Some(client) = logging.get("client") {
                if let Some(file) = client.get("file") {
                    if let Some(xml_url) = file.get("url").and_then(|u| u.as_str()) {
                        let xml_path = version_path.join("client-1.12.xml");
                        match download_file(xml_url.to_string(), xml_path).await {
                            Ok(info) => {
                                println!("✅ 日志配置文件下载成功: {} -> {}", info.url, info.path.display());
                                success_count += 1;
                            }
                            Err(e) => {
                                println!("❌ 日志配置文件下载失败: {}", e);
                                failed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // 创建两个异步任务，分别处理资源索引文件和libraries
        let assets_future = async {
            let assets_start = std::time::Instant::now();
            let mut result: Result<(), Box<dyn std::error::Error + Send + Sync>> = Ok(());

            if let Some(asset_index) = json_value.get("assetIndex") {
                if let Some(asset_url) = asset_index["url"].as_str() {
                    // 直接解析资源索引文件内容
                    let response = request::Request::new(asset_url.to_string());
                    let asset_content = response.fetch_get().await?;
                    let asset_json: serde_json::Value = serde_json::from_str(&asset_content)?;

                    // 保存资源索引文件
                    let assets_index_path = paths.assets_dir.join("indexes").join(format!("{}.json", version_id));
                    if let Some(parent) = assets_index_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&assets_index_path, &asset_content)?;
                    println!("✅ 资源索引文件已保存到: {}", assets_index_path.display());

                    if let Some(objects) = asset_json.get("objects") {
                        let paths = MinecraftPaths::new();
                        let assets_path = paths.assets_dir;
                        std::fs::create_dir_all(&assets_path)?;

                        // 准备下载任务
                        let download_tasks: Vec<(String, std::path::PathBuf, String)> = objects
                            .as_object()
                            .unwrap()
                            .iter()
                            .filter_map(|(_, value)| {
                                let hash = value.get("hash").and_then(|h| h.as_str())?;
                                let hash_prefix = &hash[..2];
                                let download_url = format!(
                                    "https://resources.download.minecraft.net/{}/{}",
                                    hash_prefix, hash
                                );
                                let object_path = assets_path.join("objects").join(hash_prefix).join(hash);

                                if let Some(parent) = object_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }

                                Some((download_url, object_path, hash.to_string()))
                            })
                            .collect();

                        let total_files = download_tasks.len();
                        let progress = DownloadProgress::new(total_files);
                        let failed_downloads = Arc::new(Mutex::new(Vec::new()));

                        println!("🚀 开始下载 {} 个资源文件...", total_files);

                        let batch_size = 250; // 控制并发量
                        let semaphore = Arc::new(tokio::sync::Semaphore::new(batch_size));

                        for chunk in download_tasks.chunks(batch_size) {
                            let mut futures = Vec::new();

                            for (url, path, expected_hash) in chunk {
                                let progress = progress.clone();
                                let failed_downloads = failed_downloads.clone();
                                let url = url.clone();
                                let path = path.clone();
                                let expected_hash = expected_hash.clone();
                                let permit = semaphore.clone().acquire_owned().await.unwrap();

                                futures.push(async move {
                                    let _permit = permit;
                                    let result = download_and_verify_file(
                                        url.clone(),
                                        path.clone(),
                                        &expected_hash,
                                        Some(progress.clone()),
                                        3,
                                    )
                                    .await;

                                    if let Err(e) = result {
                                        let mut failed = failed_downloads.lock().unwrap();
                                        failed.push((url, path));
                                        eprintln!("❌ 下载或验证失败: {}", e);
                                    }
                                });
                            }

                            // 使用stream进行并发控制
                            stream::iter(futures)
                                .buffer_unordered(batch_size) // 控制并发数
                                .collect::<Vec<_>>()
                                .await;

                            // 显示进度
                            let current = progress.get_current();
                            let total = progress.total.load(Ordering::SeqCst);
                            println!(
                                "📊 下载进度: {}/{} ({}%)",
                                current,
                                total,
                                (current as f32 / total as f32 * 100.0) as u32
                            );
                        }

                        // 处理失败的下载
                        let retry_list = failed_downloads.lock().unwrap().clone();
                        if !retry_list.is_empty() {
                            println!("🔄 重试 {} 个失败的下载...", retry_list.len());
                            for (url, path) in retry_list {
                                if let Err(e) =
                                    download_file_with_retry(url.clone(), path.clone(), None, 5)
                                        .await
                                {
                                    eprintln!("❌ 最终失败: {} -> {}", url, e);
                                    progress.update_failed();
                                } else {
                                    progress.update_success();
                                }
                            }
                        }

                        // 输出最终统计
                        let final_success = progress.success.load(Ordering::SeqCst);
                        let final_failed = progress.failed.load(Ordering::SeqCst);
                        println!("📊 下载完成:");
                        println!("✅ 成功: {} 个文件", final_success);
                        println!("❌ 失败: {} 个文件", final_failed);

                        if final_failed > 0 {
                            return Err("部分资源文件下载失败".into());
                        }

                        // 在资源下载完成后记录耗时
                        let duration = assets_start.elapsed();
                        timings.push(("资源索引文件".to_string(), duration));
                        println!(
                            "✅ 资源文件下载完成 (耗时: {:.2}秒)",
                            duration.as_secs_f64()
                        );
                    }
                }
            }

            result
        };

        let libraries_future = async {
            let libs_start = std::time::Instant::now();
            let mut success_count = 0;
            let mut failed_count = 0;

            if let Some(libraries) = json_value.get("libraries") {
                if let Some(libraries_array) = libraries.as_array() {
                    // 存储需要解压的文件信息
                    let natives_to_extract = Arc::new(Mutex::new(Vec::new()));
                    
                    // 2.下载库文件
                    let download_tasks: Vec<_> = libraries_array
                        .iter()
                        .filter_map(|library| {
                            let downloads = library.get("downloads")?;
                            let mut is_native = false;

                            // 检查是否需要解压（通过rules判断）
                            if let Some(rules) = library.get("rules") {
                                if let Some(rules_array) = rules.as_array() {
                                    if let Some(first_rule) = rules_array.first() {
                                        if let Some(os) = first_rule.get("os") {
                                            if let Some(name) = os.get("name").and_then(|n| n.as_str()) {
                                                if name == current_os {
                                                    // 如果rules第一项的os.name匹配当前系统，标记为需要解压
                                                    is_native = true;
                                                    println!("📦 发现需要解压的natives库: {}", 
                                                        library.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // 根据是否需要解压选择不同的下载源
                            let artifact = if is_native {
                                // 处理natives库
                                let natives_key = match current_os.as_str() {
                                    "windows" => "natives-windows",
                                    "osx" => "natives-macos",
                                    "linux" => "natives-linux",
                                    _ => return None,
                                };
                                
                                if let Some(classifiers) = downloads.get("classifiers") {
                                    if let Some(native_download) = classifiers.get(natives_key) {
                                        native_download
                                    } else {
                                        downloads.get("artifact")?
                                    }
                                } else {
                                    downloads.get("artifact")?
                                }
                            } else {
                                downloads.get("artifact")?
                            };

                            let url = artifact["url"].as_str()?;
                            let path = artifact.get("path").and_then(|p| p.as_str())?;
                            let sha1 = artifact["sha1"].as_str()?;
                            let library_path = paths.libraries_dir.join(path);

                            if let Some(parent) = library_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            
                            Some((url.to_string(), library_path, sha1.to_string(), is_native))
                        })
                        .collect();

                    let total_libs = download_tasks.len();
                    let progress = DownloadProgress::new(total_libs);
                    let batch_size = 50;
                    let semaphore = Arc::new(tokio::sync::Semaphore::new(batch_size));
                    let success_counter = Arc::new(AtomicUsize::new(0));
                    let failed_counter = Arc::new(AtomicUsize::new(0));

                    println!("🚀 开始下载 {} 个库文件...", total_libs);

                    // 下载库文件
                    stream::iter(download_tasks)
                        .map(|(url, path, sha1, is_native)| {
                            let semaphore = semaphore.clone();
                            let progress = progress.clone();
                            let natives_to_extract = natives_to_extract.clone();
                            let version_path = version_path.clone();
                            let version_id = version_id.to_string();
                            let success_counter = success_counter.clone();
                            let failed_counter = failed_counter.clone();

                            async move {
                                let _permit = semaphore.acquire().await.unwrap();
                                match download_and_verify_file(
                                    url.clone(),
                                    path.clone(),
                                    &sha1,
                                    Some(progress.clone()),
                                    3,
                                )
                                .await
                                {
                                    Ok(info) => {
                                        if is_native {
                                            // 将需要解压的文件信息存储起来
                                            let mut natives = natives_to_extract.lock().unwrap();
                                            natives.push((info.path.clone(), version_id.clone()));
                                            println!("✅ natives库下载成功，已加入解压队列: {}", info.path.display());
                                        }
                                        println!("✅ 库文件下载成功: {} -> {}", info.url, info.path.display());
                                        success_counter.fetch_add(1, Ordering::SeqCst);
                                    }
                                    Err(e) => {
                                        println!("❌ 库文件下载失败: {} -> {}", url, e);
                                        failed_counter.fetch_add(1, Ordering::SeqCst);
                                    }
                                }
                            }
                        })
                        .buffer_unordered(batch_size)
                        .collect::<Vec<_>>()
                        .await;

                    // 所有文件下载完成后，开始解压natives库
                    let natives = natives_to_extract.lock().unwrap().clone();
                    
                    if !natives.is_empty() {
                        println!("📦 开始解压 {} 个natives库...", natives.len());
                        
                        for (file_path, version_id) in natives {
                            let natives_dir = paths.get_natives_dir(&version_id);
                            println!("🔄 正在解压: {}", file_path.display());
                            println!("📂 解压目标目录: {}", natives_dir.display());
                            
                            // 在新线程中执行解压操作
                            if let Err(e) = tokio::task::spawn_blocking(move || {
                                if let Err(e) = std::fs::create_dir_all(&natives_dir) {
                                    println!("❌ 创建natives目录失败: {}", e);
                                    return Err(e.to_string());
                                }
                                
                                match decompression(file_path.to_str().unwrap(), &version_id) {
                                    Ok(_) => {
                                        println!("✅ natives库解压成功: {}", file_path.display());
                                        Ok(())
                                    }
                                    Err(e) => {
                                        println!("❌ natives库解压失败: {} -> {}", file_path.display(), e);
                                        Err(e.to_string())
                                    }
                                }
                            }).await.unwrap() {
                                println!("❌ 解压过程出错: {}", e);
                                failed_counter.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                        
                        println!("📦 natives库解压完成");
                    }

                    success_count = success_counter.load(Ordering::SeqCst);
                    failed_count = failed_counter.load(Ordering::SeqCst);

                    println!(
                        "📊 Libraries下载完成: 成功 {}, 失败 {}",
                        success_count, failed_count
                    );
                }
            }

            (success_count, failed_count, libs_start.elapsed())
        };

        // 修改执行顺序，先执行 libraries 下载和解压
        let libraries_result = libraries_future.await;
        let (_libs_success, _libs_failed, libs_duration) = libraries_result;
        
        // 然后执行资源索引文件下载
        let assets_result = assets_future.await;
        let _assets_result = assets_result?;

        // 添加耗时统计
        timings.push(("Libraries".to_string(), libs_duration));

        // 3. 客户端映射文件 - 直接存储在版本目录中
        if let Some(client_mappings) = json_value
            .get("downloads")
            .and_then(|downloads| downloads.get("client_mappings"))
        {
            if let Some(mapping_url) = client_mappings["url"].as_str() {
                let mapping_path = version_path.join(format!("{}-mappings.txt", version_id));
                match download_file(mapping_url.to_string(), mapping_path).await {
                    Ok(info) => {
                        println!(
                            "✅ 映射文件下载成功: {} -> {}",
                            info.url,
                            info.path.display()
                        );
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("❌ 映射文件下载失败: {}", e);
                        failed_count += 1;
                    }
                }
            }
        }

        // 输出所有资源的下载耗时统计
        println!("\n📊 下载耗时统计:");
        println!("----------------------------------------");
        for (resource, duration) in timings {
            println!("{}: {:.2}秒", resource, duration.as_secs_f64());
        }
        println!("----------------------------------------");

        println!(
            "📊 下载统计: 成功 {} 个文件, 失败 {} 个文件",
            success_count, failed_count
        );

        if failed_count > 0 {
            Err("部分文件下载失败".into())
        } else {
            Ok(json_value)
        }
    }
}

// 修改 DownloadInfo 结构体，添加下载进度跟踪
async fn download_with_progress(
    url: String,
    path: std::path::PathBuf,
    progress: Option<DownloadProgress>,
) -> Result<DownloadInfo, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    let downloaded = Arc::new(AtomicUsize::new(0));
    let file = tokio::fs::File::create(&path).await?;
    let mut writer = tokio::io::BufWriter::new(file);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
        downloaded.fetch_add(chunk.len(), Ordering::SeqCst);
    }

    writer.flush().await?;

    Ok(DownloadInfo {
        url,
        path,
        size: total_size,
        downloaded,
    })
}

// 修改重试下载函数
async fn download_file_with_retry(
    url: String,
    path: std::path::PathBuf,
    progress: Option<DownloadProgress>,
    max_retries: u32,
) -> Result<DownloadInfo, Box<dyn std::error::Error + Send + Sync>> {
    let mut retries = 0;
    let mut last_error = None;

    while retries < max_retries {
        match download_with_progress(url.clone(), path.clone(), progress.clone()).await {
            Ok(info) => {
                // 验证文件大小
                if info.size > 0 && info.downloaded.load(Ordering::SeqCst) as u64 != info.size {
                    tokio::fs::remove_file(&path).await?;
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
                return Ok(info);
            }
            Err(e) => {
                last_error = Some(e);
                retries += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "下载失败".into()))
}

// 下载文件
async fn download_file(
    url: String,
    path: std::path::PathBuf,
) -> Result<DownloadInfo, Box<dyn std::error::Error + Send + Sync>> {
    let response = request::Request::new(url.clone());
    let content = response.fetch_get().await?;

    let size = content.len() as u64;
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || std::fs::write(path_clone, content)).await??;

    Ok(DownloadInfo {
        url,
        path,
        size,
        downloaded: Arc::new(AtomicUsize::new(0)),
    })
}

// 用于文件验证
async fn download_and_verify_file(
    url: String,
    path: std::path::PathBuf,
    expected_hash: &str,
    progress: Option<DownloadProgress>,
    max_retries: u32,
) -> Result<DownloadInfo, Box<dyn std::error::Error + Send + Sync>> {
    let result =
        download_file_with_retry(url.clone(), path.clone(), progress.clone(), max_retries).await?;

    // 验证文件哈希
    let content = tokio::fs::read(&path).await?;
    let mut hasher = sha1::Sha1::new();
    hasher.update(&content);
    let actual_hash = format!("{:x}", hasher.finalize());

    if actual_hash != expected_hash {
        // 如果哈希值不匹配，删除文件并返回错误
        let _ = tokio::fs::remove_file(&path).await;
        if let Some(prog) = progress {
            prog.update_failed();
        }
        return Err(format!(
            "哈希值验证失败。期望：{}，实际：{}",
            expected_hash, actual_hash
        )
        .into());
    }

    if let Some(prog) = progress {
        prog.update_success();
    }

    Ok(result)
}

#[tokio::test]
pub async fn get_version_manifest_main() -> Result<(), String> {
    let version_manifest = Download::new(String::from(
        "https://piston-meta.mojang.com/mc/game/version_manifest.json",
    ));
    let latest_version = version_manifest.dwl_version_manifest().await.unwrap();
    println!("{}", latest_version);
    Ok(())
}

#[tokio::test]
pub async fn fetch_download_minecraft() -> Result<(), String> {
    let download = DownloadOptions::new(String::from(
        "https://piston-meta.moja…04ca1285508c/25w04a.json",
    ));
    let res = download.dwl_version_manifest().await.unwrap();
    println!("{}", res);
    Ok(())
}
