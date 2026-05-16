#[derive(Debug, Clone)]
pub struct AssetStore {
    base: std::path::PathBuf,
}

impl AssetStore {
    pub fn new(base: impl Into<std::path::PathBuf>) -> Self {
        Self { base: base.into() }
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        let path = self.base.join(name);
        std::fs::read(path)
    }
}
