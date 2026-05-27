use crate::jump::config::JumpConfig;
use crate::jump::replay::ReplayTrace;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::types::{JumpOutcome, JumpPhase};
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
    pub fn setup_event(store: &StoreRef) {
        store.jump_runtime.setup_event();
    }

    /// Create a snow system, optionally sampling snow count and wind
    /// on the very first event (Pascal-faithful one-time init).
    fn prepare_snow(store: &StoreRef) -> SnowSystem {
        let mut snow = SnowSystem::new();
        if store.jump_runtime.consume_first_event() {
            let mut rng = store.jump_runtime.rng.borrow_mut();
            let mut wind = store.jump_runtime.wind.borrow_mut();
            let snow_count = calculate_snow_count(&mut rng);
            snow.set_count(snow_count, &mut rng);
            wind.sample(&mut rng);
        }
        snow
    }

    pub fn new(
        resources: ResourcesRef,
        store: StoreRef,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
    ) -> Self {
        let snow = Self::prepare_snow(&store);
        let runner = RefCell::new(Self::build_runner(
            resources.clone(),
            &store,
            hill_idx,
            start_gate,
            participant,
            policy,
            String::new(),
            snow,
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
        let snow = Self::prepare_snow(&self.store);
        *self.runner.borrow_mut() = Self::build_runner(
            self.resources.clone(),
            &self.store,
            hill_idx,
            start_gate,
            participant,
            policy,
            phase_label,
            snow,
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

    pub fn set_hide_info_panel_text(&self, hide: bool) {
        self.runner.borrow().hide_info_panel_text.set(hide);
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

    pub fn phase(&self) -> Option<JumpPhase> {
        self.runner.borrow().phase()
    }

    pub fn frame_counter(&self) -> i32 {
        self.runner.borrow().frame_counter()
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

    /// Simulate a computer jump invisibly using the lightweight path:
    /// no `JumpRunner`, `JumpSession`, `SnowSystem`, or `ReplayRecorder` overhead.
    /// Terrain is cached in `Resources` so PCX parsing happens at most once per hill.
    pub fn simulate_hidden(&self, participant: JumpParticipant, hill_idx: usize) -> JumpOutcome {
        let terrain = self
            .resources
            .terrain(hill_idx)
            .expect("terrain must be loadable");
        let hill = self
            .resources
            .hills
            .hill(hill_idx)
            .expect("hill must exist");
        let mut rng = self.store.jump_runtime.rng.borrow_mut();
        let mut wind = self.store.jump_runtime.wind.borrow_mut();
        crate::jump::sim::simulate_computer(&participant, &terrain, hill, &mut rng, &mut wind)
    }

    pub fn elements(&self) -> Vec<Element> {
        let wind = self.store.jump_runtime.wind.borrow();
        let records = self.store.records.borrow();
        self.runner.borrow_mut().elements(JumpRunnerRenderEnv {
            font: &self.resources.font,
            langbase: &self.resources.langbase,
            hills: &self.resources.hills,
            records: &records,
            wind: &wind,
        })
    }

    /// Advance physics, AI, and wind by one frame for the visible runner.
    pub fn update(&self) {
        let mut rng = self.store.jump_runtime.rng.borrow_mut();
        let mut wind = self.store.jump_runtime.wind.borrow_mut();
        self.runner.borrow_mut().update(&mut rng, &mut wind);
    }

    pub fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(runner) = self.runner.try_borrow() {
            runner.apply_palette(palette);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_runner(
        resources: ResourcesRef,
        store: &StoreRef,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
        phase_label: String,
        snow: SnowSystem,
    ) -> JumpRunner {
        let hill = resources.hills.hill(hill_idx).cloned();
        let terrain = resources.terrain(hill_idx).map(|t| (*t).clone());
        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);
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
