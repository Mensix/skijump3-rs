pub struct AssetStore;

impl AssetStore {
    pub fn read(name: &str) -> Result<Vec<u8>, std::io::Error> {
        let path = std::path::Path::new("game/assets").join(name);
        std::fs::read(path)
    }
}
