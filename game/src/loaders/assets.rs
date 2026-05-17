use crate::data::records::{HillCatalog, RecordStore};
use crate::parsers::hills::HillBaseParser;
use crate::parsers::names::NamesParser;
use crate::parsers::records::RecordsParser;
use crate::parsers::AssetParser;

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

    pub fn load_hills(&self, name: &str) -> Result<HillCatalog, String> {
        let data = self.read(name).map_err(|e| e.to_string())?;
        HillBaseParser::parse(&data).map_err(|e| e.to_string())
    }

    pub fn load_records(&self, name: &str) -> Result<RecordStore, String> {
        let data = self.read(name).map_err(|e| e.to_string())?;
        RecordsParser::parse(&data).map_err(|e| e.to_string())
    }

    pub fn load_all_names(&self) -> Vec<String> {
        let files = &["NAMES0.SKI", "NAMES1.SKI", "NAMES2.SKI"];
        let mut all_names = Vec::new();
        for &filename in files {
            if let Ok(data) = self.read(filename) {
                if let Ok(names) = NamesParser::parse(&data) {
                    all_names.extend(names);
                }
            }
        }
        all_names
    }
}
