use crate::data::profile::ProfileStore;
use crate::data::records::{HillCatalog, RecordStore};
use crate::parsers::langbase::LangBase;
use engine::ui::Font;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub player_names: Vec<String>,
    pub hills: HillCatalog,
}

impl Resources {
    pub fn new(
        font: Font,
        langbase: Rc<LangBase>,
        player_names: Vec<String>,
        hills: HillCatalog,
    ) -> Self {
        Self {
            font,
            langbase,
            player_names,
            hills,
        }
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct Store {
    pub profiles: RefCell<ProfileStore>,
    pub records: RefCell<RecordStore>,
    pub selected_hill: RefCell<usize>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new(RecordStore::default())
    }
}

impl Store {
    pub fn new(records: RecordStore) -> Self {
        Self {
            profiles: RefCell::new(ProfileStore::new()),
            records: RefCell::new(records),
            selected_hill: RefCell::new(1),
        }
    }
}

pub type StoreRef = Rc<Store>;
