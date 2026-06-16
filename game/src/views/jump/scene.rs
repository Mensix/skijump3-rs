use crate::error::AssetError;
use crate::jump::config::JumpConfig;
use crate::jump::replay::ReplayTrace;
use crate::jump::sim;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::types::{JumpOutcome, JumpPhase, JumpTelemetry};
use crate::jump::{JumpParticipant, JumpPolicy, JumpRunner, JumpRunnerRenderEnv};
use crate::store::{GameStateRef, ResourcesRef};
use crate::views::jump::input::{JumpInputAction, JumpInputController, JumpKeyBindings};
use crate::views::replay::save_dialog::{SaveAction, SaveReplayDialog};
use engine::oxide::input::UiEvent;
use engine::oxide::PaintCx;
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
    save_dialog: RefCell<SaveReplayDialog>,
    resources: ResourcesRef,
    store: GameStateRef,
    telemetry: RefCell<Option<JumpTelemetry>>,
}

impl JumpScene {
    /// Create a snow system, optionally sampling snow count and wind
    /// on the very first event (Pascal-faithful one-time init).
    /// Also initializes wind for the event if this is the first scene.
    fn prepare_snow(store: &GameStateRef, existing: Option<SnowSystem>) -> SnowSystem {
        let mut snow = existing.unwrap_or_default();
        let is_first = store.borrow_mut().consume_first_jump_event();
        if is_first {
            let low_detail = store.borrow().config.gdetail == 1;
            {
                let guard = &mut *store.borrow_mut();
                guard
                    .wind
                    .initialize(&mut guard.rng, guard.config.windplace as u8);
                let snow_count = calculate_snow_count(&mut guard.rng);
                snow.set_count(snow_count, &mut guard.rng);
                let snow_count = if low_detail { 0 } else { snow_count };
                if snow_count == 0 {
                    snow.clear_count();
                }
                guard.wind.sample(&mut guard.rng);
            }
        }
        snow
    }

    pub fn new(
        resources: ResourcesRef,
        store: GameStateRef,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
    ) -> Self {
        let snow = Self::prepare_snow(&store, None);
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
            save_dialog: RefCell::new(SaveReplayDialog::new(resources.clone())),
            telemetry: RefCell::new(None),
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
        *self.telemetry.borrow_mut() = None;
        let existing_snow = self.runner.borrow().clone_snow();
        let snow = Self::prepare_snow(&self.store, Some(existing_snow));
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

    pub fn set_team_name(&self, name: String) {
        self.runner.borrow_mut().set_team_name(name);
    }

    pub fn set_suppress_info_panel(&self, suppress: bool) {
        self.runner.borrow().suppress_info_panel.set(suppress);
    }

    pub fn set_has_bib(&self, val: bool) {
        self.runner.borrow().set_has_bib(val);
    }

    pub fn reset_state(&self, start_gate: i32) {
        let hill_idx = self.runner.borrow().hill_idx();
        let record_distance = self
            .store
            .borrow()
            .records
            .hill_record(hill_idx)
            .map_or(0.0, |r| r.len);
        self.runner
            .borrow_mut()
            .reset_state(start_gate, record_distance);
    }

    pub fn handle_jump_input(&self, event: UiEvent) -> JumpInputAction {
        let config = &self.store.borrow().config;
        let keys = JumpKeyBindings::from_config(config);
        JumpInputController.handle_event(event, &mut self.runner.borrow_mut(), keys)
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

    /// Populate telemetry from the runner's internal jump state after a jump completes.
    /// Safe to call multiple times — only populates once.
    pub fn collect_telemetry(&self) {
        if self.telemetry.borrow().is_some() {
            return;
        }
        let runner = self.runner.borrow();
        let Some(state) = runner.state() else { return };
        let grade = state.grade.max(0) as u8;
        let height = state.height.max(0) as u8;
        let takeoff_timing = state.takeoff_counter;
        let angle_counter = state.info_counter.max(0) as u8;
        *self.telemetry.borrow_mut() = Some(JumpTelemetry::new(
            grade,
            height,
            takeoff_timing,
            angle_counter,
        ));
    }

    pub fn telemetry(&self) -> Option<JumpTelemetry> {
        *self.telemetry.borrow()
    }

    // ── replay save dialog ─────────────────────────────────────

    pub fn is_save_dialog_active(&self) -> bool {
        self.save_dialog.borrow().is_active()
    }

    pub fn open_save_dialog(&self) {
        let outcome = self.runner.borrow().outcome();
        let distance = outcome
            .map(|o| format!("{:.1}", o.distance))
            .unwrap_or_default();
        let hill_name = self
            .resources
            .hills
            .hill(self.runner.borrow().hill_idx())
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let pb = &self.store.borrow().profiles;
        let author_name = pb
            .active_order
            .first()
            .and_then(|&idx| {
                let p = pb.profiles.get(idx)?;
                Some(if p.real_name.is_empty() {
                    p.name.clone()
                } else {
                    p.real_name.clone()
                })
            })
            .unwrap_or_default();
        self.save_dialog.borrow_mut().open(
            author_name,
            format!("Huge Jump in {hill_name}"),
            distance,
            hill_name,
        );
    }

    pub fn handle_save_dialog_event(&self, event: &UiEvent) -> Option<bool> {
        let action = self.save_dialog.borrow_mut().handle_event(event);
        match action {
            Some(SaveAction::SaveReplay) => {
                if let Some(trace) = self.replay_trace() {
                    self.save_dialog.borrow_mut().write_replay(&trace);
                }
                Some(true)
            }
            Some(SaveAction::Consumed) | None => Some(false),
        }
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
        Ok({
            let mut guard = self.store.borrow_mut();
            let state = &mut *guard;
            sim::simulate_computer(
                &participant,
                &terrain,
                hill,
                &mut state.rng,
                &mut state.wind,
            )
        })
    }

    pub fn render(&self, cx: &mut PaintCx<'_>) {
        if self.is_save_dialog_active() {
            self.save_dialog.borrow().paint(cx);
            return;
        }
        let s = self.store.borrow();
        self.runner.borrow_mut().render(
            cx,
            JumpRunnerRenderEnv {
                font: &self.resources.font,
                langbase: &self.resources.langbase,
                hills: &self.resources.hills,
                records: &s.records,
                wind: &s.wind,
            },
        )
    }

    /// Advance physics, AI, and wind by one frame for the visible runner.
    pub fn update(&self) {
        if self.is_save_dialog_active() {
            return;
        }
        {
            let mut guard = self.store.borrow_mut();
            let state = &mut *guard;
            self.runner
                .borrow_mut()
                .update(&mut state.rng, &mut state.wind);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_runner(
        resources: ResourcesRef,
        store: &GameStateRef,
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
            .borrow()
            .records
            .hill_record(hill_idx)
            .map_or(0.0, |r| r.len);
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
                team_name: String::new(),
            },
            snow,
        )
    }
}
