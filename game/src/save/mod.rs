pub mod config;
pub mod crypt;
pub mod files;
pub mod players;
pub mod records;

use std::cell::RefCell;
use std::rc::Rc;

use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::parsers::langbase::LangBase;

use self::config::Config;
use self::files::FileStore;

pub trait SaveFormat {
    fn to_bytes(&self) -> Vec<u8>;
}

pub fn write_lines(out: &mut Vec<u8>, lines: &[impl AsRef<str>]) {
    for line in lines {
        out.extend(line.as_ref().as_bytes());
        out.push(b'\n');
    }
}

pub type SaveRef = Rc<SaveManager>;

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
    /// Pascal: modifies globals then calls WriteConfig at end of setupmenu.
    pub fn update_config(&self, f: impl FnOnce(&mut Config)) {
        {
            let mut config = self.config.borrow_mut();
            f(&mut config);
        }
        self.save_config();
    }

    pub fn set_language(&self, idx: usize) {
        self.langbase.selected.set(idx);
        self.config.borrow_mut().languagenumber = idx as i32;
        self.save_config();
    }

    fn save_bytes(&self, filename: &str, data: &[u8]) {
        if let Err(e) = self.files.write(filename, data) {
            eprintln!("Warning: failed to write {filename}: {e}");
        }
    }

    fn save_config(&self) {
        let config = self.config.borrow();
        if let Ok(data) = config.to_toml_bytes() {
            self.save_bytes("config.toml", &data);
        } else {
            eprintln!("Warning: failed to serialize config");
        }
    }

    fn write_to_disk<T: SaveFormat>(&self, filename: &str, data: &T) {
        self.save_bytes(filename, &data.to_bytes());
    }

    pub fn save_players(&self, store: &ProfileStore) {
        if let Ok(data) = store.to_toml_bytes() {
            self.save_bytes("players.toml", &data);
        } else {
            eprintln!("Warning: failed to serialize players");
        }
    }

    pub fn save_records(&self, store: &RecordStore) {
        self.write_to_disk("HISCORE.SKI", store);
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
