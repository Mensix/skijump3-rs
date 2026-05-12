pub struct AssetStore;

impl AssetStore {
    pub fn asset_path(name: &str) -> std::path::PathBuf {
        // Try SKIJUMP3_ASSETS env var, then exe-relative, then cwd
        if let Ok(base) = std::env::var("SKIJUMP3_ASSETS") {
            return std::path::Path::new(&base).join(name);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p = dir.join("assets").join(name);
                if p.exists() {
                    return p;
                }
            }
        }
        std::path::Path::new("assets").join(name)
    }

    pub fn read(name: &str) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(Self::asset_path(name))
    }
}
