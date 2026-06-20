use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/"]
struct Assets;

/// Centralized file IO with embedded-asset fallback for single-exe builds.
/// All file paths are relative to roots; callers use filenames like "config.toml".
#[derive(Debug, Clone)]
pub struct FileStore {
    asset_dir: PathBuf,
    save_dir: PathBuf,
}

impl FileStore {
    #[must_use]
    pub fn new(asset_dir: PathBuf, save_dir: PathBuf) -> Self {
        Self {
            asset_dir,
            save_dir,
        }
    }

    /// Read from save dir first, then embedded assets, then filesystem assets.
    #[must_use]
    pub fn read(&self, name: &str) -> Vec<u8> {
        let save_path = self.save_dir.join(name);
        match std::fs::read(&save_path) {
            Ok(data) => data,
            Err(_) => {
                if let Some(embedded) = Assets::get(name) {
                    return embedded.data.to_vec();
                }
                let asset_path = self.asset_dir.join(name);
                std::fs::read(&asset_path).unwrap_or_default()
            }
        }
    }

    /// Atomically write to save dir. Creates `save_dir` and any subdirectory
    /// in `name` if they do not exist.
    pub fn write(&self, name: &str, data: &[u8]) {
        let path = self.save_dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let tmp_path = {
            let parent = path.parent().unwrap_or(&self.save_dir);
            let file_name = path.file_name().unwrap_or_default();
            parent.join(format!(".{}.tmp", file_name.to_string_lossy()))
        };
        std::fs::write(&tmp_path, data).unwrap();
        std::fs::rename(&tmp_path, &path).unwrap();
    }

    /// Check if file exists in save dir.
    #[must_use]
    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    pub fn delete_save(&self, name: &str) {
        let _ = std::fs::remove_file(self.save_dir.join(name));
    }

    /// List filenames in save dir with a given extension (without leading dot).
    #[must_use]
    pub fn list_by_ext(&self, ext: &str) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir, ext).unwrap_or_default()
    }

    /// List filenames in a save dir subdirectory with a given extension.
    #[must_use]
    pub fn list_save_subdir_by_ext(
        &self,
        subdir: &str,
        ext: &str,
    ) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir.join(subdir), ext).unwrap_or_default()
    }

    /// List filenames from both save dir and asset dir, deduplicated and sorted.
    #[must_use]
    pub fn list_by_ext_all(&self, ext: &str) -> Vec<String> {
        let mut names = BTreeSet::new();
        names.extend(self.list_by_ext(ext));
        names.extend(self.list_by_ext_in(&self.asset_dir, ext).unwrap_or_default());
        names.into_iter().collect()
    }

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
        assert!(!data.is_empty(), "should read from embedded");
    }
}
