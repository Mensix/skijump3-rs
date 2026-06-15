use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumpPolicy;
use crate::jump::types::JumpOutcome;
use crate::route::RouteTarget;
use crate::store::{HasRuntime, ResourcesRef, Store, StoreRef};
use crate::views::jump::competition::overlay::{CompetitionOverlay, OverlayKind};
use crate::views::jump::competition::session::{CompetitionSession, SessionError};
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

pub(crate) fn drive<R, E>(
    runtime: &mut R,
    simulate_computer: &mut dyn FnMut(JumpParticipant, usize) -> Result<JumpOutcome, E>,
    mark_new_event: &mut dyn FnMut(&R::Context, bool) -> bool,
) -> Result<CompetitionFlowCommand<R::Context, R::ResultsKind>, E>
where
    R: CompetitionRuntime,
{
    loop {
        match runtime.decide_next_runtime() {
            CompetitionDecision::ShowResults(kind) => {
                return Ok(CompetitionFlowCommand::ShowResults(kind));
            }
            CompetitionDecision::Done => return Ok(CompetitionFlowCommand::Done),
            CompetitionDecision::Jump {
                participant,
                hill_idx,
                context,
                is_human,
                is_new_event,
            } => {
                if is_human {
                    return Ok(CompetitionFlowCommand::HumanJump {
                        participant,
                        hill_idx,
                        is_new_event: mark_new_event(&context, is_new_event),
                        context,
                    });
                }

                let outcome = simulate_computer(participant, hill_idx)?;
                runtime.record_jump_runtime(&context, outcome);
                if runtime.is_complete_runtime() {
                    return Ok(CompetitionFlowCommand::Done);
                }
            }
        }
    }
}

/// Shared helper: rebuild or update the jump scene for a human jump.
/// Prevents recreating the scene every frame (which would reset the jumper).
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_human_jump(
    scene: &mut Option<JumpScene>,
    ui_state: &CompetitionUiState,
    resources: &ResourcesRef,
    store: &StoreRef,
    participant: JumpParticipant,
    hill_idx: usize,
    phase_label: String,
    team_name: Option<String>,
) {
    if scene.is_none() {
        ui_state.reset_acknowledged();
        ui_state.reset_outcome_recorded();
        let new_scene = JumpScene::new(
            ResourcesRef::clone(resources),
            StoreRef::clone(store),
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
    } else if let Some(ref s) = scene {
        prepare_human_jump_scene(s, ui_state, participant, hill_idx, phase_label, team_name);
    }
}

pub(crate) fn prepare_human_jump_scene(
    scene: &JumpScene,
    ui_state: &CompetitionUiState,
    participant: JumpParticipant,
    hill_idx: usize,
    phase_label: String,
    team_name: Option<String>,
) {
    if needs_human_jump_scene_rebuild(scene, ui_state, &participant, hill_idx) {
        ui_state.reset_acknowledged();
        ui_state.reset_outcome_recorded();
        scene.rebuild_for_competition(hill_idx, 15, participant, phase_label);
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
    result: Result<Option<CompetitionFlowCommand<C, R>>, SessionError>,
) -> Option<CompetitionFlowCommand<C, R>> {
    match result {
        Ok(command) => command,
        Err(e) => {
            ui_state.enter_error(e.to_string());
            None
        }
    }
}

pub(crate) fn record_acknowledged_human_jump<R>(
    session: &CompetitionSession,
    ui_state: &CompetitionUiState,
    scene: Option<&JumpScene>,
) -> bool
where
    R: CompetitionRuntime + 'static,
    Store: HasRuntime<R>,
{
    if !ui_state.is_result_acknowledged() || ui_state.is_outcome_recorded() {
        return false;
    }
    let Some(scene) = scene else {
        return false;
    };
    if !session.record_finished_human_jump::<R>(scene) {
        return false;
    }
    ui_state.mark_outcome_recorded();
    true
}

pub(crate) fn render_jump_scene_with_overlay(
    cx: &mut PaintCx<'_>,
    scene: &JumpScene,
    overlay: &CompetitionOverlay,
    ui_state: &CompetitionUiState,
) {
    if scene.outcome().is_some() {
        scene.collect_telemetry();
    }
    let overlay_ctx = overlay.context(
        scene.phase(),
        scene.frame_counter(),
        ui_state,
        scene.telemetry(),
    );
    scene.set_suppress_info_panel(
        overlay_ctx
            .as_ref()
            .is_some_and(|ctx| ctx.kind != OverlayKind::None),
    );
    scene.render(cx);
    if let Some(ctx) = overlay_ctx {
        overlay.render(cx, &ctx);
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

pub(crate) fn handle_save_dialog(scene: &JumpScene, event: &UiEvent) -> bool {
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
    scene: &JumpScene,
    event: UiEvent,
    consume_other_actions: bool,
) -> JumpInputResult {
    let action = scene.handle_jump_input(event);
    match action {
        JumpInputAction::SaveReplay => {
            scene.open_save_dialog();
            JumpInputResult::Consumed
        }
        JumpInputAction::RouteBack => JumpInputResult::Route(RouteTarget::Back),
        JumpInputAction::None => JumpInputResult::None,
        _ if consume_other_actions => JumpInputResult::Consumed,
        _ => JumpInputResult::None,
    }
}

pub(crate) fn handle_jump_scene_event(
    scene: &JumpScene,
    ui_state: &CompetitionUiState,
    event: UiEvent,
    consume_other_actions: bool,
    accepts_only_enter_escape: bool,
    acknowledge_only_unrecorded: bool,
) -> JumpInputResult {
    if handle_save_dialog(scene, &event) {
        return JumpInputResult::Consumed;
    }

    match handle_competition_jump_input(scene, event, consume_other_actions) {
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
