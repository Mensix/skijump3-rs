use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumpPolicy;
use crate::jump::types::JumpOutcome;
use crate::route::RouteTarget;
use crate::store::{HasRuntime, ResourcesRef, Store, StoreRef};
use crate::views::jump::competition::overlay::{CompetitionOverlay, OverlayKind};
use crate::views::jump::competition::session::{CompetitionSession, SessionError};
use crate::views::jump::competition::ui_state::CompetitionUiState;
use crate::views::jump::input::{JumpInputAction, JumpInputController};
use crate::views::jump::scene::JumpScene;
use engine::ui::{Element, Event, Key};

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
    let needs_build = scene.as_ref().is_none_or(|s| {
        s.participant_id() != participant.id
            || s.hill_idx() != hill_idx
            || ui_state.is_outcome_recorded()
            || (s.outcome().is_some() && ui_state.is_result_acknowledged())
    });

    if needs_build {
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
        s.set_phase_label(phase_label);
        if let Some(name) = team_name {
            s.set_team_name(name);
        }
    }
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
    scene: &JumpScene,
    overlay: &CompetitionOverlay,
    ui_state: &CompetitionUiState,
) -> Vec<Element> {
    let overlay_ctx = overlay.context(scene.phase(), scene.frame_counter(), ui_state);
    scene.set_suppress_info_panel(
        overlay_ctx
            .as_ref()
            .is_some_and(|ctx| ctx.kind != OverlayKind::None),
    );
    let mut els = scene.elements();
    if let Some(ctx) = overlay_ctx {
        els.extend(overlay.render_elements(&ctx));
    }
    els
}

pub(crate) fn acknowledge_finished_jump(
    scene: &JumpScene,
    ui_state: &CompetitionUiState,
    event: Event,
    accepts_only_enter_escape: bool,
) -> bool {
    if scene.outcome().is_none() || ui_state.is_result_acknowledged() {
        return false;
    }
    let accepted = if accepts_only_enter_escape {
        matches!(event, Event::Keyboard(Key::Enter | Key::Escape))
    } else {
        matches!(event, Event::Keyboard(_))
    };
    if accepted {
        ui_state.acknowledge_outcome();
    }
    true
}

pub(crate) fn handle_save_dialog(scene: &JumpScene, event: &Event) -> bool {
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
    event: Event,
    consume_other_actions: bool,
) -> JumpInputResult {
    let mut session = scene.session_mut();
    match JumpInputController.handle_event(event, &mut session) {
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
    event: Event,
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

pub(crate) fn route_error_back(ui_state: &CompetitionUiState, event: Event) -> Option<RouteTarget> {
    if ui_state.render_mode() != crate::views::jump::competition::ui_state::RenderMode::Error {
        return None;
    }
    matches!(event, Event::Keyboard(_)).then_some(RouteTarget::Back)
}
