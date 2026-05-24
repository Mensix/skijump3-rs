use std::rc::Rc;

use crate::content::ContentStore;
use crate::data::records::{HillCatalog, RecordStore};
use crate::parsers::hills::HillBaseParser;
use crate::parsers::records::RecordsParser;
use crate::parsers::AssetParser;
use crate::save::files::FileStore;

#[derive(Debug, Clone)]
pub struct AssetStore {
    files: Rc<FileStore>,
}

impl AssetStore {
    pub fn new(files: Rc<FileStore>) -> Self {
        Self { files }
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        self.files.read(name)
    }

    pub fn load_hills(&self, name: &str) -> Result<HillCatalog, String> {
        let data = self.read(name).map_err(|e| e.to_string())?;
        HillBaseParser::parse(&data).map_err(|e| e.to_string())
    }

    pub fn load_records(&self, name: &str) -> Result<RecordStore, String> {
        let data = self.read(name).map_err(|e| e.to_string())?;
        RecordsParser::parse(&data).map_err(|e| e.to_string())
    }

    pub fn parse<P: AssetParser>(&self, name: &str) -> Result<P::Output, String> {
        let data = self.read(name).map_err(|e| e.to_string())?;
        P::parse(&data).map_err(|e| e.to_string())
    }

    /// Load all content from the TOML content manifest.
    pub fn load_content(&self, manifest_path: &str) -> Result<ContentStore, String> {
        ContentStore::load(&self.files, manifest_path)
    }
}
