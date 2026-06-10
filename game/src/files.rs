use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

    /// Atomically write to save dir. Creates `save_dir` if it does not
    /// exist (e.g. after first-launch setup).
    pub fn write(&self, name: &str, data: &[u8]) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.save_dir)?;
        let path = self.save_dir.join(name);
        let tmp_path = self.save_dir.join(format!(".{name}.tmp"));
        std::fs::write(&tmp_path, data)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// Check if file exists in save dir.
    #[must_use]
    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    /// List filenames in save dir with a given extension (without leading dot).
    pub fn list_by_ext(&self, ext: &str) -> Result<Vec<String>, std::io::Error> {
        self.list_by_ext_in(&self.save_dir, ext)
    }

    /// List filenames from both save dir and asset dir, deduplicated and sorted.
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
        assert!(Assets::get("sprites/manifest.toml").is_some());
    }

    #[test]
    fn file_store_falls_back_to_embedded() {
        let store = FileStore::new(PathBuf::from("/nonexistent"), PathBuf::from("/nonexistent"));
        let data = store.read("languages/english.toml");
        assert!(data.is_ok(), "should read from embedded: {:?}", data.err());
    }
}
