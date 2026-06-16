pub mod config;
pub mod players;
pub mod records;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::text::lang::LangBase;

use self::config::Config;
use crate::files::FileStore;

pub type SaveRef = Rc<SaveManager>;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Serialization(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Utf8(e) => write!(f, "Invalid UTF-8: {e}"),
            Self::Serialization(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Utf8(e) => Some(e),
            Self::Serialization(_) => None,
        }
    }
}

/// Deserialize a TOML save file from raw bytes.
pub(crate) fn parse_toml<T>(data: &[u8]) -> Result<T, SaveError>
where
    T: serde::de::DeserializeOwned,
{
    let text = std::str::from_utf8(data).map_err(SaveError::Utf8)?;
    toml::from_str(text).map_err(|e| SaveError::Serialization(e.to_string()))
}

#[derive(Debug)]
pub struct SaveManager {
    pub files: Rc<FileStore>,
    profiles_loaded: RefCell<bool>,
}

pub fn load_initial_config(files: &FileStore, langbase: &Rc<LangBase>) -> Config {
    let config = match files.read("config.toml") {
        Ok(bytes) => match Config::from_toml_bytes(&bytes) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: failed to parse config.toml: {e}");
                Config::default()
            }
        },
        Err(_) => Config::default(),
    };

    if config.languagenumber >= 0 && (config.languagenumber as usize) < langbase.languages.len()
    {
        langbase.selected.set(config.languagenumber as usize);
    }
    config
}

impl SaveManager {
    pub fn new(files: Rc<FileStore>) -> Self {
        Self {
            files,
            profiles_loaded: RefCell::new(false),
        }
    }

    pub fn save_config(&self, config: &Config) -> Result<(), SaveError> {
        let data = config.to_toml_bytes()?;
        self.files.write("config.toml", &data).map_err(SaveError::Io)
    }

    fn save_bytes(&self, filename: &str, data: &[u8]) -> Result<(), SaveError> {
        self.files.write(filename, data).map_err(SaveError::Io)?;
        Ok(())
    }

    pub fn save_players(&self, store: &ProfileStore) -> Result<(), SaveError> {
        let data = store.to_toml_bytes()?;
        self.save_bytes("players.toml", &data)
    }

    pub fn save_records(&self, store: &RecordStore) -> Result<(), SaveError> {
        let data = store.to_toml_bytes()?;
        self.save_bytes("hiscores.toml", &data)
    }

    /// Load profiles from players.toml (save then asset fallback).
    pub fn load_players(&self) -> ProfileStore {
        *self.profiles_loaded.borrow_mut() = true;
        match self.files.read("players.toml") {
            Ok(data) => match ProfileStore::from_toml_bytes(&data) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("Warning: failed to parse players.toml: {e}");
                    ProfileStore::new()
                }
            },
            Err(_) => ProfileStore::new(),
        }
    }
}
