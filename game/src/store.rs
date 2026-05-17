use crate::competition::machine::Competition;
use crate::data::hill_profile::HillTerrain;
use crate::data::profile::ProfileStore;
use crate::data::records::{HillCatalog, RecordStore};
use crate::jump::replay::ReplayTrace;
use crate::loaders::assets::AssetStore;
use crate::parsers::langbase::LangBase;
use crate::rng::Random;
use crate::jump::wind::Wind;
use engine::ui::Font;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub player_names: Vec<String>,
    pub hills: HillCatalog,
    assets: AssetStore,
    hill_cache: RefCell<HashMap<usize, Rc<HillTerrain>>>,
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
            hill_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn hill_terrain(&self, hill_idx: usize) -> Result<Rc<HillTerrain>, String> {
        let mut cache = self.hill_cache.borrow_mut();
        if let Some(terrain) = cache.get(&hill_idx) {
            return Ok(Rc::clone(terrain));
        }
        let info = self
            .hills
            .hill(hill_idx)
            .ok_or_else(|| format!("Hill {hill_idx} not found"))?;
        let terrain = Rc::new(HillTerrain::load(&self.assets, info)?);
        cache.insert(hill_idx, Rc::clone(&terrain));
        Ok(terrain)
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct PracticeState {
    pub selected_hill: Cell<usize>,
    pub start_gate: Cell<i32>,
}

impl Default for PracticeState {
    fn default() -> Self {
        Self {
            selected_hill: Cell::new(1),
            start_gate: Cell::new(15),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    pub profiles: RefCell<ProfileStore>,
    pub records: RefCell<RecordStore>,
    pub rng: RefCell<Random>,
    pub wind: RefCell<Wind>,
    pub wind_place: Cell<u8>,
    pub practice: PracticeState,
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
            practice: PracticeState::default(),
            competition: RefCell::new(None),
            selected_hill: Cell::new(1),
            start_gate: Cell::new(15),
            first_event: Cell::new(true),
            selected_replay: RefCell::new(None),
            selected_main_menu: Cell::new(0),
        }
    }
}

pub type StoreRef = Rc<Store>;
