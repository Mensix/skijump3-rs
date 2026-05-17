pub mod config;
pub mod crypt;
pub mod players;
pub mod records;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::parsers::langbase::LangBase;

use self::config::Config;

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

pub struct SaveManager {
    pub config: RefCell<Config>,
    langbase: Rc<LangBase>,
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf, langbase: Rc<LangBase>) -> Self {
        let config = Self::load_initial_config(&save_dir, &langbase);
        Self {
            config: RefCell::new(config),
            langbase,
            save_dir,
        }
    }

    fn load_initial_config(save_dir: &std::path::Path, langbase: &Rc<LangBase>) -> Config {
        let path = save_dir.join("CONFIG.SKI");
        let config = std::fs::read(&path)
            .ok()
            .and_then(|bytes| Config::parse(&bytes).ok())
            .or_else(|| {
                std::fs::read("game/assets/CONFIG.SKI")
                    .ok()
                    .and_then(|bytes| Config::parse(&bytes).ok())
            })
            .unwrap_or_default();

        if config.languagenumber >= 0 && (config.languagenumber as usize) < langbase.languages.len()
        {
            langbase.selected.set(config.languagenumber as usize);
        }
        config
    }

    pub fn set_language(&self, idx: usize) {
        self.langbase.selected.set(idx);
        self.config.borrow_mut().languagenumber = idx as i32;
        self.save_config();
    }

    fn write_to_disk<T: SaveFormat>(&self, filename: &str, data: &T) {
        let bytes = data.to_bytes();
        let path = self.save_dir.join(filename);
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("Warning: failed to write {filename}: {e}");
        }
    }

    fn save_config(&self) {
        let config = self.config.borrow();
        self.write_to_disk("CONFIG.SKI", &*config);
    }

    pub fn save_players(&self, store: &ProfileStore) {
        self.write_to_disk("PLAYERS.SKI", store);
    }

    pub fn save_records(&self, store: &RecordStore) {
        self.write_to_disk("HISCORE.SKI", store);
    }
}
