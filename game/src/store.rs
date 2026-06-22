use crate::competition::ActiveCompetition;
use crate::content::names::NameCatalog;
use crate::data::hill::HillCatalog;
use crate::data::hill_profile::HillTerrain;
use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::jump::replay::ReplayTrace;
use crate::jump::types::DEFAULT_START_GATE;
use crate::jump::wind::Wind;
use crate::rng::Random;
use crate::save::config::Config;
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
}

impl Resources {
    pub fn new(
        font: Font,
        langbase: Rc<LangBase>,
        namesets: NameCatalog,
        hills: HillCatalog,
        files: Rc<FileStore>,
    ) -> Self {
        Self {
            font,
            langbase,
            namesets,
            hills,
            terrain_cache: RefCell::new(HashMap::new()),
            files,
        }
    }

    pub fn player_names(&self, name_set_index: usize) -> &[String] {
        self.namesets.names_for_config(name_set_index as i32)
    }

    pub(crate) fn terrain(&self, hill_idx: usize) -> Rc<HillTerrain> {
        let hill = self.hills.hill(hill_idx).unwrap();
        let terrain_id = &hill.terrain_id;
        let mut cache = self.terrain_cache.borrow_mut();
        if let Some(terrain) = cache.get(terrain_id) {
            return terrain.clone();
        }
        let terrain = HillTerrain::load_with_markers(&self.files, terrain_id, hill.kr, hill.pk());
        let terrain = Rc::new(terrain);
        cache.insert(terrain_id.clone(), terrain.clone());
        terrain
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone)]
pub struct GameState {
    pub config: Config,
    pub rng: Random,
    pub wind: Wind,
    pub first_event: bool,
    pub practice_hill: usize,
    pub practice_start_gate: i32,
    pub selected_replay: Option<ReplayTrace>,
    pub active_competition: Option<ActiveCompetition>,
    pub profiles: ProfileStore,
    pub records: RecordStore,
    pub active_cup_filename: Option<String>,
    pub nav_edit_hill: Option<String>,
}

impl GameState {
    pub fn new(records: RecordStore, profiles: ProfileStore, config: Config) -> Self {
        Self {
            config,
            rng: Random::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as u32,
            ),
            wind: Wind::default(),
            first_event: true,
            practice_hill: 0,
            practice_start_gate: DEFAULT_START_GATE,
            selected_replay: None,
            active_competition: None,
            profiles,
            records,
            active_cup_filename: None,
            nav_edit_hill: None,
        }
    }

    pub fn start_active(&mut self, comp: ActiveCompetition) {
        self.wind.set_enabled(true);
        self.active_competition = Some(comp);
        self.active_cup_filename = None;
    }

    pub fn setup_jump_event(&mut self) {
        self.first_event = true;
        self.wind
            .initialize(&mut self.rng, self.config.wind_position as u8);
    }

    pub fn consume_first_jump_event(&mut self) -> bool {
        let prev = self.first_event;
        self.first_event = false;
        prev
    }

    pub fn reset_practice_wind(&mut self) {
        self.wind
            .initialize(&mut self.rng, self.config.wind_position as u8);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(
            RecordStore::default(),
            ProfileStore::default(),
            Config::default(),
        )
    }
}
