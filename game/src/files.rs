#[cfg(not(target_arch = "wasm32"))]
use std::collections::BTreeSet;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/"]
struct Assets;

/// Centralized file IO with embedded-asset fallback for single-exe builds.
/// All file paths are relative to roots; callers use filenames like "config.toml".
#[derive(Debug, Clone)]
pub struct FileStore {
    #[cfg(not(target_arch = "wasm32"))]
    asset_dir: PathBuf,
    #[cfg(not(target_arch = "wasm32"))]
    save_dir: PathBuf,
}

impl FileStore {
    #[must_use]
    pub fn new(asset_dir: PathBuf, save_dir: PathBuf) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (asset_dir, save_dir);
            Self {}
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                asset_dir,
                save_dir,
            }
        }
    }

    /// Read from save dir first, then embedded assets, then filesystem assets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        let save_path = self.save_dir.join(name);
        match std::fs::read(&save_path) {
            Ok(data) => Ok(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if let Some(embedded) = Assets::get(name) {
                    return Ok(embedded.data.to_vec());
                }
                let asset_path = self.asset_dir.join(name);
                std::fs::read(&asset_path)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        Assets::get(name)
            .map(|embedded| embedded.data.to_vec())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, name.to_string()))
    }

    /// Atomically write to save dir. Creates `save_dir` and any subdirectory
    /// in `name` if they do not exist.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write(&self, name: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let path = self.save_dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp_path = {
            let parent = path.parent().unwrap_or(&self.save_dir);
            let file_name = path.file_name().unwrap_or_default();
            parent.join(format!(".{}.tmp", file_name.to_string_lossy()))
        };
        std::fs::write(&tmp_path, data)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write(&self, _name: &str, _data: &[u8]) -> Result<(), std::io::Error> {
        Ok(())
    }

    /// Check if file exists in save dir.
    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    #[must_use]
    #[cfg(target_arch = "wasm32")]
    pub const fn exists_save(&self, _name: &str) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn delete_save(&self, name: &str) -> Result<(), std::io::Error> {
        std::fs::remove_file(self.save_dir.join(name))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn delete_save(&self, _name: &str) -> Result<(), std::io::Error> {
        Ok(())
    }

    /// List filenames in save dir with a given extension (without leading dot).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_by_ext(&self, ext: &str) -> Result<Vec<String>, std::io::Error> {
        self.list_by_ext_in(&self.save_dir, ext)
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    pub fn list_by_ext(&self, _ext: &str) -> Result<Vec<String>, std::io::Error> {
        Ok(Vec::new())
    }

    /// List filenames in a save dir subdirectory with a given extension.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_save_subdir_by_ext(
        &self,
        subdir: &str,
        ext: &str,
    ) -> Result<Vec<String>, std::io::Error> {
        match self.list_by_ext_in(&self.save_dir.join(subdir), ext) {
            Ok(names) => Ok(names),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn list_save_subdir_by_ext(
        &self,
        _subdir: &str,
        _ext: &str,
    ) -> Result<Vec<String>, std::io::Error> {
        Ok(Vec::new())
    }

    /// List filenames from both save dir and asset dir, deduplicated and sorted.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_by_ext_all(&self, ext: &str) -> Result<Vec<String>, std::io::Error> {
        let mut names = BTreeSet::new();
        if let Ok(save_names) = self.list_by_ext(ext) {
            names.extend(save_names);
        }
        if let Ok(asset_names) = self.list_by_ext_in(&self.asset_dir, ext) {
            names.extend(asset_names);
        }
        Ok(names.into_iter().collect())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn list_by_ext_all(&self, _ext: &str) -> Result<Vec<String>, std::io::Error> {
        Ok(Vec::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn list_by_ext_in(&self, dir: &Path, ext: &str) -> Result<Vec<String>, std::io::Error> {
        let mut result = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == ext) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_assets_accessible() {
        assert!(Assets::get("languages/english.toml").is_some());
    }

    #[test]
    fn file_store_falls_back_to_embedded() {
        let store = FileStore::new(PathBuf::from("/nonexistent"), PathBuf::from("/nonexistent"));
        let data = store.read("languages/english.toml");
        assert!(data.is_ok(), "should read from embedded: {:?}", data.err());
    }
}
