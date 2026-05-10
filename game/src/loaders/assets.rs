pub struct AssetStore;

impl AssetStore {
    pub fn asset_path(name: &str) -> std::path::PathBuf {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        assets.join(name)
    }

    pub fn read(name: &str) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(Self::asset_path(name))
    }
}