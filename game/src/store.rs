use crate::competition::machine::Competition;
use crate::data::profile::ProfileStore;
use crate::data::records::{HillCatalog, RecordStore};
use crate::jump::replay::ReplayTrace;
use crate::jump::types::DEFAULT_START_GATE;
use crate::jump::wind::Wind;
use crate::loaders::assets::AssetStore;
use crate::parsers::langbase::LangBase;
use crate::rng::Random;
use engine::ui::Font;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub player_names: Vec<String>,
    pub hills: HillCatalog,
    pub(crate) assets: AssetStore,
}

impl Resources {
    #[must_use]
    pub fn new(
        font: Font,
        langbase: Rc<LangBase>,
        player_names: Vec<String>,
        hills: HillCatalog,
        assets: AssetStore,
    ) -> Self {
        Self {
            font,
            langbase,
            player_names,
            hills,
            assets,
        }
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct Store {
    pub profiles: RefCell<ProfileStore>,
    pub records: RefCell<RecordStore>,
    pub rng: RefCell<Random>,
    pub wind: RefCell<Wind>,
    pub wind_place: Cell<u8>,
    pub practice_selected_hill: Cell<usize>,
    pub practice_start_gate: Cell<i32>,
    pub(crate) competition: RefCell<Option<Competition>>,
    pub selected_hill: Cell<usize>,
    pub start_gate: Cell<i32>,
    pub first_event: Cell<bool>,
    pub selected_replay: RefCell<Option<ReplayTrace>>,
    pub selected_main_menu: Cell<usize>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new(RecordStore::default())
    }
}

impl Store {
    #[must_use]
    pub fn new(records: RecordStore) -> Self {
        Self {
            profiles: RefCell::new(ProfileStore::new()),
            records: RefCell::new(records),
            rng: RefCell::new(Random::default()),
            wind: RefCell::new(Wind::default()),
            wind_place: Cell::new(0),
            practice_selected_hill: Cell::new(0),
            practice_start_gate: Cell::new(DEFAULT_START_GATE),
            competition: RefCell::new(None),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            first_event: Cell::new(true),
            selected_replay: RefCell::new(None),
            selected_main_menu: Cell::new(0),
        }
    }
}

pub type StoreRef = Rc<Store>;
