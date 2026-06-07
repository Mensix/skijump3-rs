use crate::competition::machine::Competition;
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
use crate::save::SaveRef;
use crate::text::lang::LangBase;
use engine::ui::Font;
use std::cell::{Cell, RefCell};
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
    pub(crate) fn terrain(&self, hill_idx: usize) -> Result<Rc<HillTerrain>, String> {
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
    pub(crate) rng: RefCell<Random>,
    pub(crate) wind: RefCell<Wind>,
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
}

impl Default for JumpRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Practice-mode hill and start gate settings.
#[derive(Debug, Clone)]
pub struct PracticeSettings {
    pub hill: Cell<usize>,
    pub start_gate: Cell<i32>,
}

impl PracticeSettings {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
        }
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

    pub fn with<R>(&self, f: impl FnOnce(&Competition) -> R) -> R {
        f(self
            .inner
            .borrow()
            .as_ref()
            .expect("competition not started"))
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Competition) -> R) -> R {
        f(self
            .inner
            .borrow_mut()
            .as_mut()
            .expect("competition not started"))
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

    pub fn is_some(&self) -> bool {
        self.inner.borrow().is_some()
    }
}

impl Default for CompetitionSlot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    pub jump_runtime: JumpRuntime,
    pub practice: PracticeSettings,
    pub replay_selection: ReplaySelection,
    pub competition: CompetitionSlot,
    pub profiles: RefCell<ProfileStore>,
    pub records: RefCell<RecordStore>,
    pub selected_hill: Cell<usize>,
    pub start_gate: Cell<i32>,
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
            practice: PracticeSettings::new(),
            replay_selection: ReplaySelection::new(),
            competition: CompetitionSlot::new(),
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
            profiles: RefCell::new(profiles),
            records: RefCell::new(records),
            selected_hill: Cell::new(0),
            start_gate: Cell::new(DEFAULT_START_GATE),
            selected_main_menu: Cell::new(0),
        }
    }
}

pub type StoreRef = Rc<Store>;
