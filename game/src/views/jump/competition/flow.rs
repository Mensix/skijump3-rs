use crate::jump::config::JumpParticipant;
use crate::jump::JumpPolicy;
use crate::screen::Persistence;
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{Key, UiEvent};
use crate::views::jump::competition::overlay::{
    CompetitionOverlay, OverlayKind, SceneOverlaySnapshot,
};
use crate::views::jump::competition::ui_state::CompetitionUiState;
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::scene::JumpScene;

#[derive(Debug)]
pub(crate) enum CompetitionFlowCommand<C, R> {
    HumanJump {
        participant: JumpParticipant,
        hill_idx: usize,
        context: C,
    },
    ShowResults(R),
    Done,
}

pub(crate) struct HumanJumpSetup {
    pub participant: JumpParticipant,
    pub hill_idx: usize,
    pub phase_label: String,
    pub team_name: Option<String>,
}

pub(crate) fn handle_human_jump(
    scene: &mut Option<JumpScene>,
    ui_state: &mut CompetitionUiState,
    resources: &ResourcesRef,
    state: &mut GameState,
    setup: HumanJumpSetup,
) {
    let HumanJumpSetup {
        participant,
        hill_idx,
        phase_label,
        team_name,
    } = setup;
    if scene.is_none() {
        ui_state.reset_acknowledged();
        ui_state.reset_outcome_recorded();
        let mut new_scene = JumpScene::new(
            ResourcesRef::clone(resources),
            state,
            hill_idx,
            15,
            participant,
            JumpPolicy::competition(),
        );
        new_scene.set_phase_label(phase_label);
        if let Some(name) = team_name {
            new_scene.set_team_name(name);
        }
        *scene = Some(new_scene);
    } else if let Some(s) = scene {
        prepare_human_jump_scene(
            s,
            ui_state,
            participant,
            hill_idx,
            phase_label,
            team_name,
            state,
        );
    }
}

pub(crate) fn prepare_human_jump_scene(
    scene: &mut JumpScene,
    ui_state: &mut CompetitionUiState,
    participant: JumpParticipant,
    hill_idx: usize,
    phase_label: String,
    team_name: Option<String>,
    state: &mut GameState,
) {
    if needs_human_jump_scene_rebuild(scene, ui_state, &participant, hill_idx) {
        ui_state.reset_acknowledged();
        ui_state.reset_outcome_recorded();
        scene.rebuild_for_competition(hill_idx, 15, participant, phase_label, state);
    } else {
        scene.set_phase_label(phase_label);
    }
    if let Some(name) = team_name {
        scene.set_team_name(name);
    }
}

fn needs_human_jump_scene_rebuild(
    scene: &JumpScene,
    ui_state: &CompetitionUiState,
    participant: &JumpParticipant,
    hill_idx: usize,
) -> bool {
    scene.participant_id() != participant.id
        || scene.hill_idx() != hill_idx
        || ui_state.is_outcome_recorded()
        || (scene.outcome().is_some() && ui_state.is_result_acknowledged())
}

pub(crate) fn render_jump_scene_with_overlay(
    cx: &mut dyn UiCanvas,
    scene: &mut JumpScene,
    overlay: &CompetitionOverlay,
    ui_state: &CompetitionUiState,
    state: &GameState,
) {
    if scene.is_save_dialog_active() {
        scene.render(cx, state);
        return;
    }
    if scene.outcome().is_some() {
        scene.collect_telemetry();
    }
    let overlay_ctx = overlay.context(
        ui_state,
        state,
        SceneOverlaySnapshot {
            scene_phase: scene.phase(),
            frame_counter: scene.frame_counter(),
            telemetry: scene.telemetry(),
            outcome: scene.outcome(),
            participant_id: scene.participant_id(),
            team_name: scene.team_name(),
        },
    );
    scene.set_suppress_info_panel(
        overlay_ctx
            .as_ref()
            .is_some_and(|ctx| ctx.kind != OverlayKind::None),
    );
    scene.render(cx, state);
    if let Some(ctx) = overlay_ctx {
        overlay.render(cx, &ctx, state);
    }
}

pub(crate) fn acknowledge_finished_jump(
    scene: &JumpScene,
    ui_state: &mut CompetitionUiState,
    event: UiEvent,
    accepts_only_enter_escape: bool,
) -> bool {
    if scene.outcome().is_none() || ui_state.is_result_acknowledged() {
        return false;
    }
    let accepted = if accepts_only_enter_escape {
        matches!(event, UiEvent::KeyDown(Key::Enter | Key::Escape))
    } else {
        matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_))
    };
    if accepted {
        ui_state.acknowledge_outcome();
    }
    true
}

pub(crate) fn handle_save_dialog(scene: &mut JumpScene, event: &UiEvent, cx: &Persistence) -> bool {
    if !scene.is_save_dialog_active() {
        return false;
    }
    scene.handle_save_dialog_event(event, cx);
    true
}

pub(crate) enum JumpInputResult {
    None,
    Consumed,
    OpenSetup,
}

pub(crate) fn handle_competition_jump_input(
    scene: &mut JumpScene,
    ui_state: &mut CompetitionUiState,
    event: UiEvent,
    consume_other_actions: bool,
    state: &mut GameState,
) -> JumpInputResult {
    let action = scene.handle_jump_input(state, event);
    match action {
        JumpInputAction::SaveReplay => {
            scene.open_save_dialog();
            JumpInputResult::Consumed
        }
        JumpInputAction::Aborted => {
            ui_state.acknowledge_outcome();
            JumpInputResult::Consumed
        }
        JumpInputAction::OpenSetup => JumpInputResult::OpenSetup,
        JumpInputAction::ResetWind => {
            state.reset_practice_wind();
            JumpInputResult::Consumed
        }
        JumpInputAction::None => JumpInputResult::None,
        _ if consume_other_actions => JumpInputResult::Consumed,
        _ => JumpInputResult::None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpEventOptions {
    pub consume_other_actions: bool,
    pub accepts_only_enter_escape: bool,
    pub acknowledge_only_unrecorded: bool,
}

pub(crate) fn handle_jump_scene_event(
    scene: &mut JumpScene,
    ui_state: &mut CompetitionUiState,
    event: UiEvent,
    options: JumpEventOptions,
    state: &mut GameState,
    cx: &Persistence,
) -> JumpInputResult {
    if handle_save_dialog(scene, &event, cx) {
        return JumpInputResult::Consumed;
    }

    if scene.is_save_replay_event(event) {
        scene.open_save_dialog();
        return JumpInputResult::Consumed;
    }

    if (!options.acknowledge_only_unrecorded || !ui_state.is_outcome_recorded())
        && acknowledge_finished_jump(scene, ui_state, event, options.accepts_only_enter_escape)
    {
        return JumpInputResult::Consumed;
    }

    match handle_competition_jump_input(
        scene,
        ui_state,
        event,
        options.consume_other_actions,
        state,
    ) {
        JumpInputResult::None => {}
        result => return result,
    }

    JumpInputResult::None
}
