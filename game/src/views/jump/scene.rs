use crate::error::AssetError;
use crate::jump::config::JumpConfig;
use crate::jump::replay::ReplayTrace;
use crate::jump::sim;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::types::{JumpOutcome, JumpPhase};
use crate::jump::{JumpParticipant, JumpPolicy, JumpRunner, JumpRunnerRenderEnv, JumpSession};
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::Element;
use std::cell::RefCell;

#[derive(Debug, thiserror::Error)]
pub enum JumpSceneError {
    #[error("hill {0} does not exist in catalog")]
    MissingHill(usize),

    #[error("failed to load terrain for hill {hill_idx}: {msg}")]
    Terrain { hill_idx: usize, msg: AssetError },
}

pub struct JumpScene {
    runner: RefCell<JumpRunner>,
    resources: ResourcesRef,
    store: StoreRef,
}

impl JumpScene {
    /// Create a snow system, optionally sampling snow count and wind
    /// on the very first event (Pascal-faithful one-time init).
    fn prepare_snow(store: &StoreRef) -> SnowSystem {
        let mut snow = SnowSystem::new();
        if store.consume_first_jump_event() {
            store.with_jump_rng_wind_mut(|rng, wind| {
                let snow_count = calculate_snow_count(rng);
                snow.set_count(snow_count, rng);
                wind.sample(rng);
            });
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
            .records()
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
    /// Terrain is cached in `Resources` after loading from generated assets.
    /// Returns an error if the hill catalog or terrain is unavailable.
    pub fn simulate_hidden(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
    ) -> Result<JumpOutcome, JumpSceneError> {
        let terrain = self
            .resources
            .terrain(hill_idx)
            .map_err(|msg| JumpSceneError::Terrain { hill_idx, msg })?;
        let hill = self
            .resources
            .hills
            .hill(hill_idx)
            .ok_or(JumpSceneError::MissingHill(hill_idx))?;
        Ok(self.store.with_jump_rng_wind_mut(|rng, wind| {
            sim::simulate_computer(&participant, &terrain, hill, rng, wind)
        }))
    }

    pub fn elements(&self) -> Vec<Element> {
        let records = self.store.records();
        self.store.with_jump_wind(|wind| {
            self.runner.borrow_mut().elements(JumpRunnerRenderEnv {
                font: &self.resources.font,
                langbase: &self.resources.langbase,
                hills: &self.resources.hills,
                records: &records,
                wind,
            })
        })
    }

    /// Advance physics, AI, and wind by one frame for the visible runner.
    pub fn update(&self) {
        self.store
            .with_jump_rng_wind_mut(|rng, wind| self.runner.borrow_mut().update(rng, wind));
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
            .records()
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
