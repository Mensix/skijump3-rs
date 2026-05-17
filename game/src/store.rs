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

/// Runtime state for jump physics, AI, and wind.
/// Grouped to keep related global mutation together.
#[derive(Debug, Clone)]
pub struct JumpRuntime {
    pub(crate) rng: RefCell<Random>,
    pub(crate) wind: RefCell<Wind>,
    wind_place: Cell<u8>,
    first_event: Cell<bool>,
}

impl JumpRuntime {
    pub fn new() -> Self {
        Self {
            rng: RefCell::new(Random::default()),
            wind: RefCell::new(Wind::default()),
            wind_place: Cell::new(0),
            first_event: Cell::new(true),
        }
    }

    /// Start a new competition event: mark first_event and init wind.
    /// Pascal: Tuuli.Alusta(windplace) + first-event tracking.
    pub fn setup_event(&self) {
        self.first_event.set(true);
        let mut rng = self.rng.borrow_mut();
        let mut wind = self.wind.borrow_mut();
        wind.initialize(&mut rng, self.wind_place.get());
    }

    /// Check and clear the first-event flag in one atomic step.
    #[must_use]
    pub fn consume_first_event(&self) -> bool {
        self.first_event.replace(false)
    }

    /// Reset wind for practice mode (F5).
    pub fn reset_practice_wind(&self) {
        let mut rng = self.rng.borrow_mut();
        let mut wind = self.wind.borrow_mut();
        wind.initialize(&mut rng, self.wind_place.get());
    }

    pub fn set_wind_place(&self, val: u8) {
        self.wind_place.set(val);
    }

    pub fn wind_place(&self) -> u8 {
        self.wind_place.get()
    }
}

impl Default for JumpRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    pub jump_runtime: JumpRuntime,
    pub profiles: RefCell<ProfileStore>,
    pub records: RefCell<RecordStore>,
    pub practice_selected_hill: Cell<usize>,
    pub practice_start_gate: Cell<i32>,
    pub(crate) competition: RefCell<Option<Competition>>,
    pub selected_hill: Cell<usize>,
    pub start_gate: Cell<i32>,
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
            jump_runtime: JumpRuntime::new(),
            profiles: RefCell::new(ProfileStore::new()),
            records: RefCell::new(records),
            practice_selected_hill: Cell::new(0),
            practice_start_gate: Cell::new(DEFAULT_START_GATE),
            competition: RefCell::new(None),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            selected_replay: RefCell::new(None),
            selected_main_menu: Cell::new(0),
        }
    }
}

pub type StoreRef = Rc<Store>;
