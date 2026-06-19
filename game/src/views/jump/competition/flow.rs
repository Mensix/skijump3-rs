use crate::jump::config::JumpParticipant;
use crate::jump::JumpPolicy;
use crate::route::RouteTarget;
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::competition::overlay::{CompetitionOverlay, OverlayKind};
use crate::views::jump::competition::ui_state::CompetitionUiState;
use crate::views::jump::competition::ui_state::RenderMode;
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::scene::JumpScene;
use engine::oxide::input::{Key, UiEvent};
use engine::oxide::PaintCx;

#[derive(Debug)]
pub(crate) enum CompetitionFlowCommand<C, R> {
    HumanJump {
        participant: JumpParticipant,
        hill_idx: usize,
        context: C,
        is_new_event: bool,
    },
    ShowResults(R),
    Done,
}

/// Shared helper: rebuild or update the jump scene for a human jump.
/// Prevents recreating the scene every frame (which would reset the jumper).
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_human_jump(
    scene: &mut Option<JumpScene>,
    ui_state: &CompetitionUiState,
    resources: &ResourcesRef,
    state: &mut GameState,
    participant: JumpParticipant,
    hill_idx: usize,
    phase_label: String,
    team_name: Option<String>,
) {
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
    ui_state: &CompetitionUiState,
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

pub(crate) fn command_or_error<C, R>(
    ui_state: &CompetitionUiState,
    result: Result<Option<CompetitionFlowCommand<C, R>>, impl std::fmt::Display>,
) -> Option<CompetitionFlowCommand<C, R>> {
    match result {
        Ok(command) => command,
        Err(e) => {
            ui_state.enter_error(e.to_string());
            None
        }
    }
}

pub(crate) fn render_jump_scene_with_overlay(
    cx: &mut PaintCx<'_>,
    scene: &mut JumpScene,
    overlay: &CompetitionOverlay,
    ui_state: &CompetitionUiState,
    state: &GameState,
) {
    if scene.outcome().is_some() {
        scene.collect_telemetry();
    }
    let overlay_ctx = overlay.context(
        scene.phase(),
        scene.frame_counter(),
        ui_state,
        scene.telemetry(),
        state,
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
    ui_state: &CompetitionUiState,
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

pub(crate) fn handle_save_dialog(scene: &mut JumpScene, event: &UiEvent) -> bool {
    if !scene.is_save_dialog_active() {
        return false;
    }
    scene.handle_save_dialog_event(event);
    true
}

pub(crate) enum JumpInputResult {
    None,
    Consumed,
    Route(RouteTarget),
}

pub(crate) fn handle_competition_jump_input(
    scene: &mut JumpScene,
    event: UiEvent,
    consume_other_actions: bool,
    state: &GameState,
) -> JumpInputResult {
    let action = scene.handle_jump_input(state, event);
    match action {
        JumpInputAction::SaveReplay => {
            scene.open_save_dialog(state);
            JumpInputResult::Consumed
        }
        JumpInputAction::RouteBack => JumpInputResult::Route(RouteTarget::Back),
        JumpInputAction::None => JumpInputResult::None,
        _ if consume_other_actions => JumpInputResult::Consumed,
        _ => JumpInputResult::None,
    }
}

pub(crate) fn handle_jump_scene_event(
    scene: &mut JumpScene,
    ui_state: &CompetitionUiState,
    event: UiEvent,
    consume_other_actions: bool,
    accepts_only_enter_escape: bool,
    acknowledge_only_unrecorded: bool,
    state: &GameState,
) -> JumpInputResult {
    if handle_save_dialog(scene, &event) {
        return JumpInputResult::Consumed;
    }

    match handle_competition_jump_input(scene, event, consume_other_actions, state) {
        JumpInputResult::None => {}
        result => return result,
    }

    if (!acknowledge_only_unrecorded || !ui_state.is_outcome_recorded())
        && acknowledge_finished_jump(scene, ui_state, event, accepts_only_enter_escape)
    {
        return JumpInputResult::Consumed;
    }

    JumpInputResult::None
}

pub(crate) fn route_error_back(
    ui_state: &CompetitionUiState,
    event: UiEvent,
) -> Option<RouteTarget> {
    if ui_state.render_mode() != RenderMode::Error {
        return None;
    }
    matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)).then_some(RouteTarget::Back)
}
