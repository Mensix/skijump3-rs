use crate::components::modal::Modal;
use crate::data::hill::FALLBACK_HILL_FILENAME;
use crate::data::hill_profile::HillProfileMismatch;
use crate::data::profile::Profile;
use crate::jump::types::{JumpOutcome, JumpPhase};
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenEventCx, UiEvent};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::profile_updates::apply_profile_jump_side_effects;
use crate::views::jump::scene::JumpScene;

pub struct TrainingJumpView {
    scene: JumpScene,
    coach: CompetitionOverlay,
    resources: ResourcesRef,
    outcome_recorded: bool,
    profile_mismatch: Option<HillProfileMismatch>,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, state: &mut GameState) -> Self {
        let hill_idx = state.practice_hill;
        let participant = JumpParticipant::trainee();
        let start_gate = state.practice_start_gate;
        let profile_mismatch = resources.verify_hill_profile(hill_idx);
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            state,
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );

        Self {
            scene,
            coach: CompetitionOverlay::new(resources.clone()),
            resources,
            outcome_recorded: false,
            profile_mismatch,
        }
    }

    const fn has_profile_mismatch(&self) -> bool {
        self.profile_mismatch.is_some()
    }

    fn handle_jump_event(
        &mut self,
        state: &mut GameState,
        nav: &mut ScreenEventCx<RouteTarget>,
        event: UiEvent,
    ) {
        let action = self.scene.handle_jump_input(state, event);
        match action {
            JumpInputAction::None => {}
            JumpInputAction::SaveReplay => {
                self.scene.open_save_dialog();
            }
            JumpInputAction::ResetWind => {
                state.reset_practice_wind();
            }
            JumpInputAction::ResetJump => {
                self.scene.prepare_training_attempt_weather(state);
                self.scene.reset_state(state, state.practice_start_gate);
                self.outcome_recorded = false;
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                state.practice_start_gate = start_gate;
            }
            JumpInputAction::OpenSetup => nav.navigate(RouteTarget::OptionsMenu),
            JumpInputAction::Consumed => {}
            JumpInputAction::Aborted => return_to_training_setup(nav),
        }
    }

    fn record_completed_jump(&mut self, state: &mut GameState) {
        if self.outcome_recorded {
            return;
        }
        let Some(outcome) = self.scene.outcome() else {
            return;
        };
        self.outcome_recorded = true;
        let Some(profile_idx) = current_trainee_profile(state) else {
            return;
        };
        let hill_idx = self.scene.hill_idx();
        let (hill_file, hill_display) = self.resources.hills.hill(hill_idx).map_or_else(
            || (FALLBACK_HILL_FILENAME.to_string(), String::new()),
            |hill| (hill.terrain_id.clone(), hill.name.clone()),
        );
        if let Some(profile) = state.profiles.profiles.get_mut(profile_idx) {
            apply_training_result(
                profile,
                hill_idx,
                hill_file,
                hill_display,
                self.scene.phase(),
                outcome,
            );
        }
    }

    fn paint_content(&mut self, cx: &mut dyn UiCanvas, state: &GameState) {
        if let Some(mismatch) = &self.profile_mismatch {
            Modal::hill_profile_mismatch(&mismatch.front_index, mismatch.exiting_cup)
                .paint(cx, &self.resources.langbase);
            return;
        }
        if self.scene.outcome().is_some() {
            self.scene.collect_telemetry();
        }
        self.scene.render(cx, state);
        let coach_style = current_trainee_profile(state)
            .and_then(|idx| state.profiles.profiles.get(idx))
            .map_or(0, |profile| profile.coach_style as u8);
        if !self.scene.is_save_dialog_active() && self.scene.phase() == Some(JumpPhase::Result) {
            if let Some(telemetry) = self.scene.telemetry() {
                self.coach.render_coach_feedback(cx, telemetry, coach_style);
            }
        }
    }

    fn handle_input(
        &mut self,
        state: &mut GameState,
        cx: &Persistence,
        nav: &mut ScreenEventCx<RouteTarget>,
        event: UiEvent,
    ) {
        if self.scene.is_save_dialog_active() {
            self.scene.handle_save_dialog_event(&event, cx);
        } else {
            self.handle_jump_event(state, nav, event)
        }
    }
}

fn return_to_training_setup(nav: &mut ScreenEventCx<RouteTarget>) {
    nav.back();
}

fn current_trainee_profile(state: &GameState) -> Option<usize> {
    state
        .profiles
        .active_order
        .first()
        .copied()
        .filter(|&idx| idx < state.profiles.profiles.len())
}

fn apply_training_result(
    profile: &mut Profile,
    hill_idx: usize,
    hill_file: String,
    hill_display: String,
    phase: Option<JumpPhase>,
    outcome: JumpOutcome,
) {
    apply_profile_jump_side_effects(
        profile,
        hill_idx,
        hill_file,
        hill_display,
        outcome,
        phase == Some(JumpPhase::Result),
        None,
    );
}

impl GameScreen for TrainingJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        if self.profile_mismatch.is_some() {
            return;
        }
        self.scene.update(cx.state);
        self.record_completed_jump(cx.state);
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.profile_mismatch.is_some() {
            match event {
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(_, _) => {
                    nav.back();
                }
                UiEvent::Quit | UiEvent::Tick => {}
            }
            nav.consume();
            return;
        }
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            UiEvent::KeyDown(Key::Escape | Key::F10)
                if !self.scene.is_save_dialog_active()
                    && !matches!(
                        self.scene.phase(),
                        Some(
                            JumpPhase::Info
                                | JumpPhase::OnBar
                                | JumpPhase::Inrun
                                | JumpPhase::Flight
                                | JumpPhase::Landing
                        )
                    ) =>
            {
                nav.back();
                return;
            }
            _ => {}
        }
        let persistence = cx.persistence();
        self.handle_input(cx.state, &persistence, nav, event);
        nav.consume();
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint, cx.state);
    }

    fn has_modal(&self) -> bool {
        self.has_profile_mismatch()
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_training_result, current_trainee_profile};
    use crate::data::profile::Profile;
    use crate::jump::types::{FallType, JumpOutcome, JumpPhase, LandingStyle};
    use crate::store::GameState;
    use crate::ui::{NavAction, ScreenEventCx};

    fn outcome(distance: f64, aborted: bool) -> JumpOutcome {
        JumpOutcome {
            distance,
            score: 100.0,
            style_points: [18.0; 5],
            landing_style: LandingStyle::Telemark,
            fall_type: FallType::None,
            injury: 0,
            aborted,
        }
    }

    #[test]
    fn valid_training_result_updates_jump_count_and_personal_best() {
        let mut profile = Profile::default();

        apply_training_result(
            &mut profile,
            3,
            "HILL3".to_string(),
            "Test Hill".to_string(),
            Some(JumpPhase::Result),
            outcome(123.5, false),
        );

        assert_eq!(profile.total_jumps, 1);
        assert_eq!(profile.best_jump, 123.5);
        assert_eq!(profile.besthill_idx, 3);
        assert_eq!(profile.best_hill_file, "HILL3");
        assert_eq!(profile.best_hill_display, "Test Hill");
    }

    #[test]
    fn training_uses_the_current_active_profile() {
        let mut state = GameState::default();
        state.profiles.profiles.push(Profile {
            coach_style: 3,
            ..Profile::default()
        });
        state.profiles.active_order = vec![1, 0];

        let profile_idx = current_trainee_profile(&state).unwrap();

        assert_eq!(profile_idx, 1);
        assert_eq!(state.profiles.profiles[profile_idx].coach_style, 3);
    }

    #[test]
    fn aborted_and_disqualified_training_attempts_do_not_update_profile() {
        for (phase, result) in [
            (Some(JumpPhase::Result), outcome(100.0, true)),
            (Some(JumpPhase::Disqualified), outcome(100.0, false)),
        ] {
            let mut profile = Profile::default();
            apply_training_result(
                &mut profile,
                3,
                "HILL3".to_string(),
                "Test Hill".to_string(),
                phase,
                result,
            );

            assert_eq!(profile.total_jumps, 0);
            assert_eq!(profile.best_jump, 0.0);
        }
    }

    #[test]
    fn aborted_training_jump_returns_to_training_setup() {
        let mut nav = ScreenEventCx::default();

        super::return_to_training_setup(&mut nav);

        assert_eq!(nav.take_action(), NavAction::Back);
    }

    #[test]
    fn shorter_valid_training_result_only_updates_jump_count() {
        let mut profile = Profile {
            total_jumps: 4,
            best_jump: 130.0,
            besthill_idx: 1,
            best_hill_file: "OLD".to_string(),
            best_hill_display: "Old Hill".to_string(),
            ..Profile::default()
        };

        apply_training_result(
            &mut profile,
            3,
            "HILL3".to_string(),
            "Test Hill".to_string(),
            Some(JumpPhase::Result),
            outcome(120.0, false),
        );

        assert_eq!(profile.total_jumps, 5);
        assert_eq!(profile.best_jump, 130.0);
        assert_eq!(profile.besthill_idx, 1);
        assert_eq!(profile.best_hill_file, "OLD");
    }
}
