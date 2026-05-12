use crate::data::profile::ProfileStore;
use crate::parsers::langbase::LangBase;
use engine::ui::Font;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Store {
    pub profiles: ProfileStore,
    pub font: Font,
    pub langbase: Rc<LangBase>,
}

impl Store {
    pub fn new(font: Font, langbase: Rc<LangBase>) -> Self {
        Self {
            profiles: ProfileStore::new(),
            font,
            langbase,
        }
    }
}

pub type StoreRef = Rc<RefCell<Store>>;
