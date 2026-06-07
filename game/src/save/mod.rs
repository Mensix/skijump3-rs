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
    Serialization(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Serialization(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serialization(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct SaveManager {
    pub config: RefCell<Config>,
    langbase: Rc<LangBase>,
    pub files: Rc<FileStore>,
    profiles_loaded: RefCell<bool>,
}

impl SaveManager {
    pub fn new(files: Rc<FileStore>, langbase: Rc<LangBase>) -> Self {
        let config = Self::load_initial_config(&files, &langbase);
        Self {
            config: RefCell::new(config),
            langbase,
            files,
            profiles_loaded: RefCell::new(false),
        }
    }

    fn load_initial_config(files: &FileStore, langbase: &Rc<LangBase>) -> Config {
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

    /// Apply a mutation to the config and persist immediately.
    /// Pascal: modifies globals then calls `WriteConfig` at end of setupmenu.
    pub fn update_config(&self, f: impl FnOnce(&mut Config)) {
        {
            let mut config = self.config.borrow_mut();
            f(&mut config);
        }
        if let Err(e) = self.save_config() {
            eprintln!("Warning: failed to save config: {e}");
        }
    }

    pub fn set_language(&self, idx: usize) {
        self.langbase.selected.set(idx);
        self.config.borrow_mut().languagenumber = idx as i32;
        if let Err(e) = self.save_config() {
            eprintln!("Warning: failed to save config: {e}");
        }
    }

    fn save_bytes(&self, filename: &str, data: &[u8]) -> Result<(), SaveError> {
        self.files
            .write(filename, data)
            .map_err(SaveError::Io)?;
        Ok(())
    }

    fn save_config(&self) -> Result<(), SaveError> {
        let config = self.config.borrow();
        let data = config
            .to_toml_bytes()
            .map_err(SaveError::Serialization)?;
        self.save_bytes("config.toml", &data)
    }

    pub fn save_players(&self, store: &ProfileStore) -> Result<(), SaveError> {
        let data = store
            .to_toml_bytes()
            .map_err(SaveError::Serialization)?;
        self.save_bytes("players.toml", &data)
    }

    pub fn save_records(&self, store: &RecordStore) -> Result<(), SaveError> {
        let data = store
            .to_toml_bytes()
            .map_err(SaveError::Serialization)?;
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
