use crate::competition::koth::types::KothRuntime;
use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;
use crate::competition::ActiveCompetition;
use crate::content::names::NameCatalog;
use crate::data::hill::HillCatalog;
use crate::data::hill_profile::HillTerrain;
use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::error::AssetError;
use crate::files::FileStore;
use crate::jump::replay::ReplayTrace;
use crate::jump::types::DEFAULT_START_GATE;
use crate::jump::wind::Wind;
use crate::rng::Random;
use crate::save::{SaveManager, SaveRef};
use crate::text::lang::LangBase;
use engine::oxide::Font;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub namesets: NameCatalog,
    pub hills: HillCatalog,
    pub(crate) terrain_cache: RefCell<HashMap<String, Rc<HillTerrain>>>,
    pub files: Rc<FileStore>,
    pub save_manager: SaveRef,
}

impl Resources {
    pub fn player_names(&self) -> &[String] {
        self.namesets
            .names_for_config(self.save_manager.config.borrow().namenumber)
    }

    pub fn new(
        font: Font,
        langbase: Rc<LangBase>,
        namesets: NameCatalog,
        hills: HillCatalog,
        files: Rc<FileStore>,
        save_manager: SaveRef,
    ) -> Self {
        Self {
            font,
            langbase,
            namesets,
            hills,
            terrain_cache: RefCell::new(HashMap::new()),
            files,
            save_manager,
        }
    }

    pub(crate) fn terrain(&self, hill_idx: usize) -> Result<Rc<HillTerrain>, AssetError> {
        let hill = self
            .hills
            .hill(hill_idx)
            .ok_or_else(|| AssetError::Custom(format!("Hill {hill_idx} not found")))?;
        let terrain_id = &hill.terrain_id;
        let mut cache = self.terrain_cache.borrow_mut();
        if let Some(terrain) = cache.get(terrain_id) {
            return Ok(terrain.clone());
        }
        let terrain = HillTerrain::load(&self.files, terrain_id)?;
        let terrain = Rc::new(terrain);
        cache.insert(terrain_id.clone(), terrain.clone());
        Ok(terrain)
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct GameState {
    pub rng: Random,
    pub wind: Wind,
    pub wind_place: u8,
    pub first_event: bool,
    pub practice_hill: usize,
    pub practice_start_gate: i32,
    pub selected_replay: Option<ReplayTrace>,
    pub active_competition: Option<ActiveCompetition>,
    pub profiles: ProfileStore,
    pub records: RecordStore,
    pub selected_main_menu: usize,
}

impl GameState {
    pub fn new(records: RecordStore, profiles: ProfileStore) -> Self {
        Self {
            rng: Random::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as u32,
            ),
            wind: Wind::default(),
            wind_place: 0,
            first_event: true,
            practice_hill: 0,
            practice_start_gate: DEFAULT_START_GATE,
            selected_replay: None,
            active_competition: None,
            profiles,
            records,
            selected_main_menu: 0,
        }
    }

    pub fn configure_from_save(&mut self, save_manager: &SaveManager) {
        self.wind_place = save_manager.config.borrow().windplace as u8;
    }

    pub fn start_active(&mut self, comp: ActiveCompetition) {
        self.wind.set_enabled(true);
        self.active_competition = Some(comp);
    }

    pub fn setup_jump_event(&mut self) {
        self.first_event = true;
        self.wind.initialize(&mut self.rng, self.wind_place);
    }

    pub fn consume_first_jump_event(&mut self) -> bool {
        let prev = self.first_event;
        self.first_event = false;
        prev
    }

    pub fn reset_practice_wind(&mut self) {
        self.wind.initialize(&mut self.rng, self.wind_place);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(RecordStore::default(), ProfileStore::new())
    }
}

pub type GameStateRef = Rc<RefCell<GameState>>;

pub trait HasRuntime<R> {
    fn with_runtime_mut<T>(&mut self, f: impl FnOnce(&mut R) -> T) -> Option<T>;
}

impl HasRuntime<Competition> for GameState {
    fn with_runtime_mut<T>(&mut self, f: impl FnOnce(&mut Competition) -> T) -> Option<T> {
        self.active_competition.as_mut()?.individual_mut().map(f)
    }
}

impl HasRuntime<TeamCupRuntime> for GameState {
    fn with_runtime_mut<T>(&mut self, f: impl FnOnce(&mut TeamCupRuntime) -> T) -> Option<T> {
        self.active_competition
            .as_mut()?
            .team_cup_runtime_mut()
            .map(f)
    }
}

impl HasRuntime<KothRuntime> for GameState {
    fn with_runtime_mut<T>(&mut self, f: impl FnOnce(&mut KothRuntime) -> T) -> Option<T> {
        self.active_competition.as_mut()?.koth_runtime_mut().map(f)
    }
}
