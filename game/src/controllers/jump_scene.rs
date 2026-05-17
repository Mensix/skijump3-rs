use crate::data::hill_profile::HillTerrain;
use crate::data::records::HillInfo;
use crate::jump::config::JumpConfig;
use crate::jump::replay::ReplayTrace;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::types::JumpOutcome;
use crate::jump::{JumpParticipant, JumpPolicy, JumpRunner, JumpRunnerRenderEnv, JumpSession};
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::Element;
use std::cell::RefCell;

pub struct JumpScene {
    runner: RefCell<JumpRunner>,
    resources: ResourcesRef,
    store: StoreRef,
}

impl JumpScene {
    /// Set up wind and first-event state for a new competition event.
    /// Pascal: Tuuli.Alusta(windplace) once per event before any jumpers.
    /// Call before constructing `JumpScene` or at the start of each event.
    pub fn setup_event(store: &StoreRef) {
        store.first_event.set(true);
        let mut rng = store.rng.borrow_mut();
        let mut wind = store.wind.borrow_mut();
        wind.initialize(&mut rng, store.wind_place.get());
    }

    pub fn new(
        resources: ResourcesRef,
        store: StoreRef,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
    ) -> Self {
        let runner = RefCell::new(Self::build_runner(
            resources.clone(),
            &store,
            hill_idx,
            start_gate,
            participant,
            policy,
            String::new(),
        ));
        Self {
            runner,
            resources,
            store,
        }
    }

    pub fn rebuild(
        &self,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
        phase_label: String,
    ) {
        *self.runner.borrow_mut() = Self::build_runner(
            self.resources.clone(),
            &self.store,
            hill_idx,
            start_gate,
            participant,
            policy,
            phase_label,
        );
    }

    pub fn rebuild_for_competition(
        &self,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        phase_label: String,
    ) {
        self.rebuild(
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::competition(),
            phase_label,
        );
    }

    pub fn set_phase_label(&self, label: String) {
        self.runner.borrow_mut().set_phase_label(label);
    }

    pub fn reset_state(&self, start_gate: i32) {
        let hill_idx = self.runner.borrow().hill_idx();
        let record_distance = self
            .store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);
        self.runner
            .borrow_mut()
            .reset_state(start_gate, record_distance);
    }

    pub fn session_mut(&self) -> impl std::ops::DerefMut<Target = JumpSession> + use<'_> {
        std::cell::RefMut::map(self.runner.borrow_mut(), |r| r.session_mut())
    }

    pub fn outcome(&self) -> Option<JumpOutcome> {
        self.runner.borrow().outcome()
    }

    pub fn replay_trace(&self) -> Option<ReplayTrace> {
        self.runner.borrow().replay_trace()
    }

    pub fn hill_idx(&self) -> usize {
        self.runner.borrow().hill_idx()
    }

    pub fn participant_id(&self) -> usize {
        self.runner.borrow().participant_id()
    }

    /// Build a temporary runner and simulate a computer jump invisibly.
    /// Does not mutate the visible runner — safe to call from a `&self`
    /// context alongside the view's own `&self` scene usage.
    pub fn simulate_hidden(&self, participant: JumpParticipant, hill_idx: usize) -> JumpOutcome {
        let mut runner =
            Self::build_hidden_runner(&self.resources, &self.store, participant, hill_idx);
        let mut rng = self.store.rng.borrow_mut();
        let mut wind = self.store.wind.borrow_mut();
        runner.simulate_to_completion(&mut rng, &mut wind)
    }

    /// Build a hidden computer-runner (no snow, no wind init).
    fn build_hidden_runner(
        resources: &ResourcesRef,
        store: &StoreRef,
        participant: JumpParticipant,
        hill_idx: usize,
    ) -> JumpRunner {
        let (hill, terrain, record_distance) = Self::load_hill_data(resources, store, hill_idx);
        JumpRunner::new(
            JumpConfig {
                hill_idx,
                hill,
                terrain,
                start_gate: 15,
                snow_count: 0,
                participant,
                policy: JumpPolicy::competition(),
                record_distance,
                phase_label: String::new(),
            },
            SnowSystem::new(),
        )
    }

    pub fn elements(&self) -> Vec<Element> {
        let mut rng = self.store.rng.borrow_mut();
        let mut wind = self.store.wind.borrow_mut();
        let records = self.store.records.borrow();
        self.runner.borrow_mut().elements(JumpRunnerRenderEnv {
            font: &self.resources.font,
            langbase: &self.resources.langbase,
            hills: &self.resources.hills,
            records: &records,
            rng: &mut rng,
            wind: &mut wind,
        })
    }

    pub fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Ok(mut runner) = self.runner.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            runner.render_snow(framebuffer, wind);
        }
    }

    pub fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(runner) = self.runner.try_borrow() {
            runner.apply_palette(palette);
        }
    }

    /// Load hill data common to both visible and hidden runner construction.
    fn load_hill_data(
        resources: &ResourcesRef,
        store: &StoreRef,
        hill_idx: usize,
    ) -> (Option<HillInfo>, Result<HillTerrain, String>, i32) {
        let hill = resources.hills.hill(hill_idx).cloned();
        let terrain = hill.as_ref().map_or_else(
            || Err(format!("Hill {hill_idx} not found")),
            |info| HillTerrain::load(&resources.assets, info),
        );
        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);
        (hill, terrain, record_distance)
    }

    fn build_runner(
        resources: ResourcesRef,
        store: &StoreRef,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
        phase_label: String,
    ) -> JumpRunner {
        let (hill, terrain, record_distance) = Self::load_hill_data(&resources, store, hill_idx);
        let mut snow = SnowSystem::new();

        if terrain.is_ok() && hill.is_some() {
            if store.first_event.get() {
                let mut rng = store.rng.borrow_mut();
                let mut wind = store.wind.borrow_mut();
                let snow_count = calculate_snow_count(&mut rng);
                snow.set_count(snow_count, &mut rng);
                wind.sample(&mut rng);
                store.first_event.set(false);
            }
        }

        let snow_count = snow.count();
        JumpRunner::new(
            JumpConfig {
                hill_idx,
                hill,
                terrain,
                start_gate,
                snow_count,
                participant,
                policy,
                record_distance,
                phase_label,
            },
            snow,
        )
    }
}
