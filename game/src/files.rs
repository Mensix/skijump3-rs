use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/"]
struct Assets;

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

    pub fn read(&self, name: &str) -> Vec<u8> {
        let asset_path = self.asset_dir.join(name);
        if let Ok(data) = std::fs::read(&asset_path) {
            return data;
        }
        if let Some(embedded) = Assets::get(name) {
            return embedded.data.to_vec();
        }
        Vec::new()
    }

    pub fn read_save(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.save_dir.join(name)).unwrap_or_default()
    }

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

    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    pub fn delete_save(&self, name: &str) {
        let _ = std::fs::remove_file(self.save_dir.join(name));
    }

    pub fn list_by_ext(&self, ext: &str) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir, ext).unwrap_or_default()
    }

    pub fn list_save_subdir_by_ext(&self, subdir: &str, ext: &str) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir.join(subdir), ext)
            .unwrap_or_default()
    }

    pub fn list_by_ext_all(&self, ext: &str) -> Vec<String> {
        let mut names = BTreeSet::new();
        names.extend(self.list_by_ext(ext));
        names.extend(
            self.list_by_ext_in(&self.asset_dir, ext)
                .unwrap_or_default(),
        );
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
