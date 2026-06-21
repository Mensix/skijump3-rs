pub mod config;
pub mod cup;
pub mod players;
pub mod records;

use std::cell::RefCell;
use std::rc::Rc;

use crate::competition::ActiveCompetition;
use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::text::lang::LangBase;

use self::config::Config;
use self::cup::{CupSaveData, CupSaveEntry};
use crate::files::FileStore;

pub type SaveRef = Rc<SaveManager>;

/// Deserialize a TOML save file from raw bytes.
pub(crate) fn parse_toml<T>(data: &[u8]) -> T
where
    T: serde::de::DeserializeOwned,
{
    let text = std::str::from_utf8(data).unwrap();
    toml::from_str(text).unwrap()
}

#[derive(Debug)]
pub struct SaveManager {
    pub files: Rc<FileStore>,
    profiles_loaded: RefCell<bool>,
}

pub fn load_initial_config(files: &FileStore, langbase: &Rc<LangBase>) -> Config {
    let bytes = files.read("config.toml");
    let config = Config::from_toml_bytes(&bytes);

    langbase.apply_saved_language(config.language);
    config
}

impl SaveManager {
    pub fn new(files: Rc<FileStore>) -> Self {
        Self {
            files,
            profiles_loaded: RefCell::new(false),
        }
    }

    pub fn save_config(&self, config: &Config) {
        let data = config.to_toml_bytes();
        self.files.write("config.toml", &data);
    }

    fn save_bytes(&self, filename: &str, data: &[u8]) {
        self.files.write(filename, data);
    }

    pub fn save_players(&self, store: &ProfileStore) {
        let data = store.to_toml_bytes();
        self.save_bytes("players.toml", &data);
    }

    pub fn save_records(&self, store: &RecordStore) {
        let data = store.to_toml_bytes();
        self.save_bytes("hiscores.toml", &data);
    }

    pub fn save_active_cup(&self, active: &ActiveCompetition) -> String {
        let filename = format!("cup_{}.toml", chrono::Local::now().format("%Y%m%d_%H%M"));
        let saved_at = current_timestamp_string();
        let data = CupSaveData::new(active.clone(), saved_at).to_toml_bytes();
        self.save_bytes(&filename, &data);
        filename
    }

    pub fn load_cup(&self, filename: &str) -> CupSaveData {
        let data = self.files.read(filename);
        CupSaveData::from_toml_bytes(&data)
    }

    pub fn list_cup_saves(&self) -> Vec<CupSaveEntry> {
        let names = self.files.list_by_ext("toml");
        let mut entries = Vec::new();
        for filename in names.into_iter().filter(|name| name.starts_with("cup_")) {
            let data = self.load_cup(&filename);
            entries.push(CupSaveEntry {
                filename,
                title: data.title(),
                saved_at: data.saved_at,
            });
        }
        entries.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
        entries
    }

    /// Load profiles from players.toml (save then asset fallback).
    pub fn load_players(&self) -> ProfileStore {
        *self.profiles_loaded.borrow_mut() = true;
        let data = self.files.read("players.toml");
        ProfileStore::from_toml_bytes(&data)
    }
}

fn current_timestamp_string() -> String {
    chrono::Local::now()
        .format("%a %d %b %Y, %H:%M")
        .to_string()
        .to_uppercase()
}
