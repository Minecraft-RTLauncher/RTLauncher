// ***
// 路径配置管理模块
// ***

use std::path::PathBuf;

pub struct MinecraftPaths {
    pub base_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub assets_dir: PathBuf,
}

impl MinecraftPaths {
    pub fn new() -> Self {
        let base_dir = PathBuf::from("D:\\Desktop\\.minecraft");
        Self {
            versions_dir: base_dir.join("version"),
            libraries_dir: base_dir.join("libraries"),
            assets_dir: base_dir.join("assets"),
            base_dir: base_dir,
        }
    }

    pub fn get_version_dir(&self, version_id: &str) -> PathBuf {
        self.versions_dir.join(version_id)
    }

    pub fn get_natives_dir(&self, version_id: &str) -> PathBuf {
        self.get_version_dir(version_id).join(format!("{}-natives", version_id))
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.versions_dir)?;
        std::fs::create_dir_all(&self.libraries_dir)?;
        std::fs::create_dir_all(&self.assets_dir)?;
        Ok(())
    }

    pub fn get_absolute_path(&self, path: PathBuf) -> String {
        path.canonicalize()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .trim_start_matches(r"\\?\")
            .to_string()
    }

    pub fn get_libraries_classpath(&self) -> Vec<String> {
        walkdir::WalkDir::new(&self.libraries_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "jar"))
            .map(|entry| self.get_absolute_path(entry.path().to_path_buf()))
            .collect()
    }
}