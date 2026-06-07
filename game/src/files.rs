use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Centralized file IO with save-dir override over asset-dir fallback.
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

    /// Read first from `save_dir`, fallback to `asset_dir` on `NotFound`.
    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        let save_path = self.save_dir.join(name);
        match std::fs::read(&save_path) {
            Ok(data) => Ok(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let asset_path = self.asset_dir.join(name);
                std::fs::read(&asset_path)
            }
            Err(e) => Err(e),
        }
    }

    /// Atomically write to save dir.
    pub fn write(&self, name: &str, data: &[u8]) -> Result<(), std::io::Error> {
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
        let dot_ext = format!(".{ext}");
        let mut result = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.len() > dot_ext.len()
                        && name[name.len() - dot_ext.len()..].eq_ignore_ascii_case(&dot_ext)
                    {
                        result.push(name.to_string());
                    }
                }
            }
        }
        result.sort();
        Ok(result)
    }

    /// Save dir path for callers that need it directly.
    #[must_use]
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> (FileStore, tempfile::TempDir, tempfile::TempDir) {
        let save = tempfile::tempdir().unwrap();
        let asset = tempfile::tempdir().unwrap();
        let store = FileStore::new(asset.path().to_path_buf(), save.path().to_path_buf());
        (store, save, asset)
    }

    #[test]
    fn read_fallback_from_asset() {
        let (store, _save, asset) = setup();
        fs::write(asset.path().join("fallback.txt"), b"asset data").unwrap();
        let data = store.read("fallback.txt").unwrap();
        assert_eq!(data, b"asset data");
    }

    #[test]
    fn save_overrides_asset() {
        let (store, save, asset) = setup();
        fs::write(asset.path().join("override.txt"), b"asset data").unwrap();
        fs::write(save.path().join("override.txt"), b"save data").unwrap();
        let data = store.read("override.txt").unwrap();
        assert_eq!(data, b"save data");
    }

    #[test]
    fn atomic_write_no_tmp_left_behind() {
        let (store, save, _asset) = setup();
        store.write("clean.txt", b"data").unwrap();
        // No .clean.txt.tmp should remain
        let entries: Vec<_> = fs::read_dir(save.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries.iter().any(|n| n.starts_with('.')));
    }

    #[test]
    fn list_by_ext_filters_by_extension() {
        let (store, save, _asset) = setup();
        fs::write(save.path().join("a.SJR"), b"").unwrap();
        fs::write(save.path().join("b.SJR"), b"").unwrap();
        fs::write(save.path().join("c.txt"), b"").unwrap();
        let mut names = store.list_by_ext("SJR").unwrap();
        names.sort();
        assert_eq!(names, vec!["a.SJR", "b.SJR"]);
    }
}
