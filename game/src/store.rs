use crate::competition::machine::Competition;
use crate::competition::runtime::CompetitionRuntime;
use crate::competition::team_cup::types::TeamCupRuntime;
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
use crate::save::SaveRef;
use crate::text::lang::LangBase;
use engine::ui::Font;
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
            rng: RefCell::new(Random::default()),
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

/// Wraps `Option<Competition>` with scoped access methods so callers
/// don't need to choreograph `borrow()` / `drop()` manually.
#[derive(Debug, Clone)]
pub struct CompetitionSlot {
    inner: RefCell<Option<Competition>>,
}

impl CompetitionSlot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub fn start(&self, comp: Competition) {
        *self.inner.borrow_mut() = Some(comp);
    }

    pub fn try_with<R>(&self, f: impl FnOnce(&Competition) -> R) -> Option<R> {
        self.inner.borrow().as_ref().map(f)
    }

    /// Pass the inner Competition to a callback that may need
    /// concurrent access to other Store fields. The borrow is
    /// released when the callback returns.
    pub fn try_with_mut<R>(&self, f: impl FnOnce(&mut Competition) -> R) -> Option<R> {
        self.inner.borrow_mut().as_mut().map(f)
    }
}

impl Default for CompetitionSlot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    jump_runtime: JumpRuntime,
    practice: PracticeSettings,
    replay_selection: ReplaySelection,
    competition: CompetitionSlot,
    team_cup: RefCell<Option<TeamCupRuntime>>,
    profiles: RefCell<ProfileStore>,
    records: RefCell<RecordStore>,
    selected_hill: Cell<usize>,
    start_gate: Cell<i32>,
    selected_main_menu: Cell<usize>,
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
            practice: PracticeSettings::new(),
            replay_selection: ReplaySelection::new(),
            competition: CompetitionSlot::new(),
            team_cup: RefCell::new(None),
            profiles: RefCell::new(ProfileStore::new()),
            records: RefCell::new(records),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            selected_main_menu: Cell::new(0),
        }
    }

    #[must_use]
    pub fn with_profiles(records: RecordStore, profiles: ProfileStore) -> Self {
        Self {
            jump_runtime: JumpRuntime::new(),
            practice: PracticeSettings::new(),
            replay_selection: ReplaySelection::new(),
            competition: CompetitionSlot::new(),
            team_cup: RefCell::new(None),
            profiles: RefCell::new(profiles),
            records: RefCell::new(records),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            selected_main_menu: Cell::new(0),
        }
    }

    pub fn start_competition(&self, comp: Competition) {
        self.competition.start(comp);
    }

    pub fn try_with_competition<R>(&self, f: impl FnOnce(&Competition) -> R) -> Option<R> {
        self.competition.try_with(f)
    }

    pub fn try_with_competition_mut<R>(&self, f: impl FnOnce(&mut Competition) -> R) -> Option<R> {
        self.competition.try_with_mut(f)
    }

    pub fn start_team_cup(&self, tc: TeamCupRuntime) {
        *self.team_cup.borrow_mut() = Some(tc);
    }

    pub fn try_with_team_cup<R>(&self, f: impl FnOnce(&TeamCupRuntime) -> R) -> Option<R> {
        self.team_cup.borrow().as_ref().map(f)
    }

    pub fn try_with_team_cup_mut<R>(&self, f: impl FnOnce(&mut TeamCupRuntime) -> R) -> Option<R> {
        self.team_cup.borrow_mut().as_mut().map(f)
    }

    pub fn clear_team_cup(&self) {
        *self.team_cup.borrow_mut() = None;
    }

    pub fn profiles(&self) -> Ref<'_, ProfileStore> {
        self.profiles.borrow()
    }

    pub fn profiles_mut(&self) -> RefMut<'_, ProfileStore> {
        self.profiles.borrow_mut()
    }

    pub fn records(&self) -> Ref<'_, RecordStore> {
        self.records.borrow()
    }

    pub fn try_records(&self) -> Option<Ref<'_, RecordStore>> {
        self.records.try_borrow().ok()
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

/// Generic access to a competition runtime in the store.
pub(crate) trait HasRuntime<R: CompetitionRuntime + 'static> {
    fn with_runtime_mut<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut R) -> T;
}

impl HasRuntime<Competition> for Store {
    fn with_runtime_mut<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Competition) -> T,
    {
        self.try_with_competition_mut(f)
    }
}

impl HasRuntime<TeamCupRuntime> for Store {
    fn with_runtime_mut<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut TeamCupRuntime) -> T,
    {
        self.try_with_team_cup_mut(f)
    }
}

pub type StoreRef = Rc<Store>;
