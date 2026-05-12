use std::rc::Rc;
use std::cell::RefCell;
use engine::ui::Font;
use crate::data::profile::ProfileStore;

pub struct Store {
    pub profiles: ProfileStore,
    pub font: Font,
}

impl Store {
    pub fn new(font: Font) -> Self {
        Self { profiles: ProfileStore::new(), font }
    }
}

pub type StoreRef = Rc<RefCell<Store>>;
