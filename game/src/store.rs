use crate::data::profile::ProfileStore;
use crate::parsers::langbase::LangBase;
use engine::ui::Font;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub player_names: Vec<String>,
}

impl Resources {
    pub fn new(font: Font, langbase: Rc<LangBase>, player_names: Vec<String>) -> Self {
        Self {
            font,
            langbase,
            player_names,
        }
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct Store {
    pub profiles: ProfileStore,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            profiles: ProfileStore::new(),
        }
    }
}

pub type StoreRef = Rc<RefCell<Store>>;
