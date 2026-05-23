use std::path::{Path, PathBuf};

/// Centralized file IO with save-dir override over asset-dir fallback.
/// All file paths are relative to roots; callers use filenames like "CONFIG.SKI".
#[derive(Debug, Clone)]
pub struct FileStore {
    asset_dir: PathBuf,
    save_dir: PathBuf,
}

impl FileStore {
    pub fn new(asset_dir: PathBuf, save_dir: PathBuf) -> Self {
        Self {
            asset_dir,
            save_dir,
        }
    }

    /// Read first from save_dir, fallback to asset_dir.
    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        let save_path = self.save_dir.join(name);
        match std::fs::read(&save_path) {
            Ok(data) => Ok(data),
            Err(_) => {
                let asset_path = self.asset_dir.join(name);
                std::fs::read(&asset_path)
            }
        }
    }

    /// Read from asset dir only (for bundled non-overridable assets).
    pub fn read_asset(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.asset_dir.join(name))
    }

    /// Read from save dir only.
    pub fn read_save(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.save_dir.join(name))
    }

    /// Atomically write to save dir.
    pub fn write(&self, name: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let path = self.save_dir.join(name);
        let tmp_path = self.save_dir.join(format!(".{}.tmp", name));
        std::fs::write(&tmp_path, data)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// Check if file exists in save dir.
    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    /// List filenames in save dir with a given extension (without leading dot).
    pub fn list_by_ext(&self, ext: &str) -> Result<Vec<String>, std::io::Error> {
        let dot_ext = format!(".{}", ext);
        let mut result = Vec::new();
        for entry in std::fs::read_dir(&self.save_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(&dot_ext) {
                        result.push(name.to_string());
                    }
                }
            }
        }
        result.sort();
        Ok(result)
    }

    /// Save dir path for callers that need it directly.
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }
}
