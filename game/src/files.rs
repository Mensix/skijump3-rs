use std::path::Path;
use std::path::PathBuf;

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
        std::fs::read(self.asset_dir.join(name)).unwrap_or_default()
    }

    pub fn read_save(&self, name: &str) -> Vec<u8> {
        let path = self.save_dir.join(name);
        std::fs::read(&path).unwrap_or_default()
    }

    pub fn read_save_or_asset(&self, name: &str) -> Vec<u8> {
        let path = self.save_dir.join(name);
        match std::fs::read(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.read(name),
            Err(_) => Vec::new(),
        }
    }

    pub fn write(&self, name: &str, data: &[u8]) -> bool {
        let path = self.save_dir.join(name);
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
        std::fs::write(&path, data).is_ok()
    }

    pub fn exists_save(&self, name: &str) -> bool {
        self.save_dir.join(name).exists()
    }

    pub fn delete_save(&self, name: &str) {
        let path = self.save_dir.join(name);
        let _ = std::fs::remove_file(&path);
    }

    pub fn list_by_ext(&self, ext: &str) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir, ext)
    }

    pub fn list_replays(&self) -> Vec<String> {
        let mut names = self.list_by_ext("SJR");
        for name in self.list_by_ext_in(&self.asset_dir, "SJR") {
            if !names
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&name))
            {
                names.push(name);
            }
        }
        names
    }

    pub fn list_save_subdir_by_ext(&self, subdir: &str, ext: &str) -> Vec<String> {
        self.list_by_ext_in(&self.save_dir.join(subdir), ext)
    }

    fn list_by_ext_in(&self, dir: &Path, ext: &str) -> Vec<String> {
        let mut result = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return result,
            Err(_) => return result,
        };
        for entry in entries {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(ext))
            {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
        result
    }
}

pub fn default_save_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            return exe_dir.to_path_buf();
        }
    }

    PathBuf::from(".")
}

pub fn find_asset_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        if from_manifest.join("content.toml").exists() {
            return from_manifest;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if exe_dir.join("content.toml").exists() {
                return exe_dir.to_path_buf();
            }
            if exe_dir.join("assets").join("content.toml").exists() {
                return exe_dir.join("assets");
            }
            return exe_dir.to_path_buf();
        }
    }

    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn file_store_reads_from_asset_dir() {
        let assets = tempdir().expect("asset dir");
        std::fs::write(assets.path().join("custom.txt"), b"external").expect("write asset");
        let store = FileStore::new(assets.path().to_path_buf(), PathBuf::from("/nonexistent"));

        assert_eq!(store.read("custom.txt"), b"external");
        assert!(store.read("missing.txt").is_empty());
    }

    #[test]
    fn extension_discovery_is_ascii_case_insensitive() {
        let saves = tempdir().expect("save dir");
        std::fs::write(saves.path().join("LOWER.sjr"), b"replay").expect("write replay");
        let store = FileStore::new(PathBuf::from("/nonexistent"), saves.path().to_path_buf());

        assert_eq!(store.list_by_ext("SJR"), vec!["LOWER.sjr"]);
    }

    #[test]
    fn save_override_precedes_asset_and_deletion_reveals_asset() {
        let assets = tempdir().expect("asset dir");
        std::fs::write(assets.path().join("INTRO.SJR"), b"base").expect("write asset");
        let saves = tempdir().expect("save dir");
        let store = FileStore::new(assets.path().to_path_buf(), saves.path().to_path_buf());
        store.write("INTRO.SJR", b"override");

        assert_eq!(store.read_save_or_asset("INTRO.SJR"), b"override");
        store.delete_save("INTRO.SJR");
        assert_eq!(store.read_save_or_asset("INTRO.SJR"), b"base");
    }

    #[test]
    fn write_replaces_existing_destination() {
        let saves = tempdir().unwrap();
        let store = FileStore::new(PathBuf::from("/nonexistent"), saves.path().to_path_buf());
        store.write("config.toml", b"old");

        assert!(store.write("config.toml", b"new"));

        assert_eq!(store.read_save("config.toml"), b"new");
    }

    #[test]
    fn missing_save_reads_empty() {
        let saves = tempdir().unwrap();
        let store = FileStore::new(PathBuf::from("/nonexistent"), saves.path().to_path_buf());
        assert!(store.read_save("missing").is_empty());
    }
}
