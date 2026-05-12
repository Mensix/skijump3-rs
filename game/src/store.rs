use std::rc::Rc;
use std::cell::RefCell;
use crate::data::profile::ProfileStore;

pub struct Store {
    pub profiles: ProfileStore,
}

impl Store {
    pub fn new() -> Self {
        Self { profiles: ProfileStore::new() }
    }
}

pub type StoreRef = Rc<RefCell<Store>>;
