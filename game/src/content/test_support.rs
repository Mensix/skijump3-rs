use crate::files::FileStore;
use std::fs;
use tempfile::TempDir;

pub(crate) fn make_files() -> (FileStore, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path().to_path_buf(), dir.path().to_path_buf());
    (store, dir)
}

pub(crate) fn write(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}
