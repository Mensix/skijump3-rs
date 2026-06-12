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
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub namesets: NameCatalog,
    pub hills: HillCatalog,
    pub(crate) terrain_cache: RefCell<HashMap<usize, Rc<HillTerrain>>>,
    pub files: Rc<FileStore>,
    pub save_manager: SaveRef,
}

impl Resources {
    #[must_use]
    pub fn player_names(&self) -> &[String] {
        self.namesets
            .names_for_config(self.save_manager.config.borrow().namenumber)
    }

    #[must_use]
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

    /// Load (or retrieve cached) terrain for a given hill index.
    /// Loads pre-converted hill data from generated assets.
    pub(crate) fn terrain(&self, hill_idx: usize) -> Result<Rc<HillTerrain>, AssetError> {
        self.hills
            .hill(hill_idx)
            .ok_or_else(|| AssetError::Custom(format!("Hill {hill_idx} not found")))?;
        let mut cache = self.terrain_cache.borrow_mut();
        if let Some(terrain) = cache.get(&hill_idx) {
            return Ok(terrain.clone());
        }
        let terrain = HillTerrain::load(&self.files, hill_idx)?;
        let terrain = Rc::new(terrain);
        cache.insert(hill_idx, terrain.clone());
        Ok(terrain)
    }
}

pub type ResourcesRef = Rc<Resources>;

/// Runtime state for jump physics, AI, and wind.
/// Grouped to keep related global mutation together.
#[derive(Debug, Clone)]
pub struct JumpRuntime {
    rng: RefCell<Random>,
    wind: RefCell<Wind>,
    wind_place: Cell<u8>,
    first_event: Cell<bool>,
}

impl JumpRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: RefCell::new(Random::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as u32,
            )),
            wind: RefCell::new(Wind::default()),
            wind_place: Cell::new(0),
            first_event: Cell::new(true),
        }
    }

    /// Start a new competition event: mark `first_event` and init wind.
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

    pub fn with_rng_wind_mut<R>(&self, f: impl FnOnce(&mut Random, &mut Wind) -> R) -> R {
        let mut rng = self.rng.borrow_mut();
        let mut wind = self.wind.borrow_mut();
        f(&mut rng, &mut wind)
    }

    pub fn with_wind<R>(&self, f: impl FnOnce(&Wind) -> R) -> R {
        let wind = self.wind.borrow();
        f(&wind)
    }
}

impl Default for JumpRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Practice-mode hill and start gate settings.
#[derive(Debug, Clone)]
pub struct PracticeSettings {
    hill: Cell<usize>,
    start_gate: Cell<i32>,
}

impl PracticeSettings {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
        }
    }

    #[must_use]
    pub fn hill(&self) -> usize {
        self.hill.get()
    }

    pub fn set_hill(&self, hill: usize) {
        self.hill.set(hill);
    }

    #[must_use]
    pub fn start_gate(&self) -> i32 {
        self.start_gate.get()
    }

    pub fn set_start_gate(&self, start_gate: i32) {
        self.start_gate.set(start_gate);
    }
}

impl Default for PracticeSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Selected replay for playback.
#[derive(Debug, Clone)]
pub struct ReplaySelection {
    inner: RefCell<Option<ReplayTrace>>,
}

impl ReplaySelection {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub fn select(&self, trace: ReplayTrace) {
        *self.inner.borrow_mut() = Some(trace);
    }

    pub fn clone_selected(&self) -> Option<ReplayTrace> {
        self.inner.borrow().clone()
    }
}

impl Default for ReplaySelection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    jump_runtime: JumpRuntime,
    practice: PracticeSettings,
    replay_selection: ReplaySelection,
    active_competition: RefCell<Option<crate::competition::ActiveCompetition>>,
    profiles: RefCell<ProfileStore>,
    records: RefCell<RecordStore>,
    selected_hill: Cell<usize>,
    start_gate: Cell<i32>,
    selected_main_menu: Cell<usize>,
}

impl Default for Store {
    fn default() -> Self {
        Self::from_loaded_data(RecordStore::default(), ProfileStore::new())
    }
}

impl Store {
    #[must_use]
    pub fn from_loaded_data(records: RecordStore, profiles: ProfileStore) -> Self {
        Self {
            jump_runtime: JumpRuntime::new(),
            practice: PracticeSettings::new(),
            replay_selection: ReplaySelection::new(),
            active_competition: RefCell::new(None),
            profiles: RefCell::new(profiles),
            records: RefCell::new(records),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            selected_main_menu: Cell::new(0),
        }
    }

    pub fn configure_from_save(&self, save_manager: &SaveManager) {
        self.set_wind_place(save_manager.config.borrow().windplace as u8);
    }

    pub fn start_active(&self, comp: crate::competition::ActiveCompetition) {
        *self.active_competition.borrow_mut() = Some(comp);
    }

    pub fn with_active<R>(
        &self,
        f: impl FnOnce(&crate::competition::ActiveCompetition) -> R,
    ) -> Option<R> {
        self.active_competition.borrow().as_ref().map(f)
    }

    pub fn with_active_mut<R>(
        &self,
        f: impl FnOnce(&mut crate::competition::ActiveCompetition) -> R,
    ) -> Option<R> {
        self.active_competition.borrow_mut().as_mut().map(f)
    }

    pub fn profiles(&self) -> Ref<'_, ProfileStore> {
        self.profiles.borrow()
    }

    pub fn profiles_mut(&self) -> RefMut<'_, ProfileStore> {
        self.profiles.borrow_mut()
    }

    pub fn with_profiles<R>(&self, f: impl FnOnce(&ProfileStore) -> R) -> R {
        let profiles = self.profiles.borrow();
        f(&profiles)
    }

    pub fn with_profiles_mut<R>(&self, f: impl FnOnce(&mut ProfileStore) -> R) -> R {
        let mut profiles = self.profiles.borrow_mut();
        f(&mut profiles)
    }

    pub fn records(&self) -> Ref<'_, RecordStore> {
        self.records.borrow()
    }

    pub fn with_records<R>(&self, f: impl FnOnce(&RecordStore) -> R) -> R {
        let records = self.records.borrow();
        f(&records)
    }

    #[must_use]
    pub fn practice_hill(&self) -> usize {
        self.practice.hill()
    }

    pub fn set_practice_hill(&self, hill: usize) {
        self.practice.set_hill(hill);
    }

    #[must_use]
    pub fn practice_start_gate(&self) -> i32 {
        self.practice.start_gate()
    }

    pub fn set_practice_start_gate(&self, start_gate: i32) {
        self.practice.set_start_gate(start_gate);
    }

    pub fn set_selected_hill(&self, hill: usize) {
        self.selected_hill.set(hill);
    }

    pub fn set_start_gate(&self, start_gate: i32) {
        self.start_gate.set(start_gate);
    }

    #[must_use]
    pub fn selected_main_menu(&self) -> usize {
        self.selected_main_menu.get()
    }

    pub fn set_selected_main_menu(&self, selected: usize) {
        self.selected_main_menu.set(selected);
    }

    pub fn setup_jump_event(&self) {
        self.jump_runtime.setup_event();
    }

    #[must_use]
    pub fn consume_first_jump_event(&self) -> bool {
        self.jump_runtime.consume_first_event()
    }

    pub fn reset_practice_wind(&self) {
        self.jump_runtime.reset_practice_wind();
    }

    pub fn set_wind_place(&self, val: u8) {
        self.jump_runtime.set_wind_place(val);
    }

    pub fn wind_place(&self) -> u8 {
        self.jump_runtime.wind_place()
    }

    pub fn with_jump_rng_wind_mut<R>(&self, f: impl FnOnce(&mut Random, &mut Wind) -> R) -> R {
        self.jump_runtime.with_rng_wind_mut(f)
    }

    pub fn with_jump_wind<R>(&self, f: impl FnOnce(&Wind) -> R) -> R {
        self.jump_runtime.with_wind(f)
    }

    pub fn select_replay(&self, trace: ReplayTrace) {
        self.replay_selection.select(trace);
    }

    pub fn clone_selected_replay(&self) -> Option<ReplayTrace> {
        self.replay_selection.clone_selected()
    }
}

pub type StoreRef = Rc<Store>;

pub trait HasRuntime<R> {
    fn with_runtime_mut<T>(&self, f: impl FnOnce(&mut R) -> T) -> Option<T>;
}

impl HasRuntime<crate::competition::machine::Competition> for Store {
    fn with_runtime_mut<T>(
        &self,
        f: impl FnOnce(&mut crate::competition::machine::Competition) -> T,
    ) -> Option<T> {
        self.with_active_mut(|active| active.individual_mut().map(f))
            .flatten()
    }
}

impl HasRuntime<crate::competition::team_cup::types::TeamCupRuntime> for Store {
    fn with_runtime_mut<T>(
        &self,
        f: impl FnOnce(&mut crate::competition::team_cup::types::TeamCupRuntime) -> T,
    ) -> Option<T> {
        self.with_active_mut(|active| active.team_cup_runtime_mut().map(f))
            .flatten()
    }
}

impl HasRuntime<crate::competition::koth::types::KothRuntime> for Store {
    fn with_runtime_mut<T>(
        &self,
        f: impl FnOnce(&mut crate::competition::koth::types::KothRuntime) -> T,
    ) -> Option<T> {
        self.with_active_mut(|active| active.koth_runtime_mut().map(f))
            .flatten()
    }
}
