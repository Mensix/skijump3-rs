use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind, TeamCupRuntime};
use crate::components::screen;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::flow::{
    acknowledge_finished_jump, handle_competition_jump_input, handle_human_jump,
    handle_save_dialog, record_acknowledged_human_jump, render_jump_scene_with_overlay,
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::results::{self as competition_results, CompetitionResultsRequest};
use crate::views::jump::competition::session::CompetitionSession;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::scene::JumpScene;
use engine::ui::{Blinker, Element, Event, Key, View};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewPhase {
    NamingTeam(usize),
    Ready,
    ShowTeams,
    Jumping,
    Done,
}

pub struct TeamCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    phase: ViewPhase,
    blinker: Blinker,
    session: CompetitionSession,
    team_names: Vec<String>,
    name_buffer: String,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let names = super::setup::team_names(&store);
        let name_buffer = names.first().cloned().unwrap_or_default();
        Self {
            resources: resources.clone(),
            store: store.clone(),
            scene: None,
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(resources.clone(), store.clone()),
            phase: ViewPhase::NamingTeam(0),
            blinker: Blinker::new(),
            session: CompetitionSession::new(resources, store),
            team_names: names,
            name_buffer,
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn naming_elements(&self) -> Vec<Element> {
        let current_team = match self.phase {
            ViewPhase::NamingTeam(idx) => idx,
            _ => 0,
        };
        super::setup::naming_elements(
            &self.resources,
            &self.store,
            &self.team_names,
            current_team,
            &self.name_buffer,
            self.cursor_visible,
        )
    }

    fn ready_elements(&self) -> Vec<Element> {
        super::setup::ready_elements(
            &self.resources,
            &self.store,
            &self.team_names,
            self.cursor_visible,
        )
    }

    fn showteams_elements(&self) -> Vec<Element> {
        super::setup::showteams_elements(&self.resources, &self.store, self.cursor_visible)
    }

    fn drive_until_visible(&mut self) {
        let scene = JumpScene::new(
            ResourcesRef::clone(&self.resources),
            StoreRef::clone(&self.store),
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        );
        match self.session.drive_competition::<TeamCupRuntime>(&scene) {
            Ok(Some(cmd)) => self.apply_command(cmd),
            Ok(None) => {
                self.ui_state.enter_error("No competition running".into());
            }
            Err(e) => {
                self.ui_state.enter_error(e.to_string());
            }
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<TeamCupJumpContext, TeamCupResultsKind>,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => {
                let phase_label = if context.round_idx == 0 {
                    self.resources.langbase.lstr(54).to_string()
                } else {
                    self.resources.langbase.lstr(55).to_string()
                };
                handle_human_jump(
                    &mut self.scene,
                    &self.ui_state,
                    &self.resources,
                    &self.store,
                    participant,
                    hill_idx,
                    phase_label,
                    Some(context.team_name.clone()),
                );
                self.ui_state.enter_jump();
            }
            CompetitionFlowCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.ui_state.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.phase = ViewPhase::Done;
                self.ui_state.enter_done();
            }
        }
    }

    fn finalize_current_name(&mut self) {
        let name = self.name_buffer.trim().to_string();
        let n = match self.phase {
            ViewPhase::NamingTeam(idx) => idx,
            _ => return,
        };

        if !name.is_empty() {
            self.store.with_active_mut(|active| {
                let Some(tc) = active.team_cup_runtime_mut() else {
                    return;
                };
                let human_indices: Vec<usize> = tc
                    .teams
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.is_human_team)
                    .map(|(i, _)| i)
                    .rev() // Pascal: GetTeam(0)→jnimet[15], GetTeam(1)→jnimet[14]
                    .collect();
                if let Some(&team_idx) = human_indices.get(n) {
                    tc.teams[team_idx].name = name.clone();
                }
            });
            self.team_names[n] = name;
        }

        self.name_buffer.clear();

        if n + 1 < self.team_names.len() {
            self.phase = ViewPhase::NamingTeam(n + 1);
            self.name_buffer = self.team_names[n + 1].clone();
        } else {
            self.phase = ViewPhase::Ready;
        }
    }
}

impl View<RouteTarget> for TeamCupJumpView {
    fn update(&mut self) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if matches!(
            self.phase,
            ViewPhase::NamingTeam(_) | ViewPhase::Ready | ViewPhase::ShowTeams
        ) {
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        record_acknowledged_human_jump::<TeamCupRuntime>(
            &self.session,
            &self.ui_state,
            self.scene.as_ref(),
        );

        // Drive competition only after human jump outcome is recorded,
        // not every frame during the jump (avoids recreating the scene).
        if self.ui_state.is_outcome_recorded() && self.ui_state.render_mode() != RenderMode::Results
        {
            if let Some(ref scene) = self.scene {
                match self.session.drive_competition::<TeamCupRuntime>(scene) {
                    Ok(Some(cmd)) => self.apply_command(cmd),
                    Ok(None) => {}
                    Err(e) => self.ui_state.enter_error(e.to_string()),
                }
            }
        }

        if let Some(ref mut scene) = self.scene {
            scene.update();
        }
    }

    fn elements(&self) -> Vec<Element> {
        if matches!(self.phase, ViewPhase::NamingTeam(_)) {
            return self.naming_elements();
        }
        if self.phase == ViewPhase::Ready {
            return self.ready_elements();
        }
        if self.phase == ViewPhase::ShowTeams {
            return self.showteams_elements();
        }
        if self.phase == ViewPhase::Done {
            return vec![];
        }

        match self.ui_state.render_mode() {
            RenderMode::Jump => {
                if let Some(ref scene) = self.scene {
                    render_jump_scene_with_overlay(scene, &self.overlay, &self.ui_state)
                } else {
                    vec![]
                }
            }
            RenderMode::Results => {
                competition_results::render(
                    &self.resources,
                    &self.store,
                    &self.ui_state,
                    CompetitionResultsRequest::TeamCup {
                        kind: self.results_kind,
                    },
                )
            }
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.ui_state.render_mode() == RenderMode::Error {
                    self.ui_state.error_message()
                } else {
                    String::new()
                };
                screen::message_screen(&msg, self.resources.langbase.lstr(15))
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if let Some(route) = route_error_back(&self.ui_state, event) {
            return Some(route);
        }
        if self.ui_state.render_mode() == RenderMode::Error {
            return None;
        }

        if self.phase == ViewPhase::Done {
            return Some(RouteTarget::Back);
        }

        if matches!(self.phase, ViewPhase::NamingTeam(_)) {
            let Event::Keyboard(key) = event;
            match key {
                Key::Char(c) if c.is_ascii_graphic() || c == ' ' => {
                    let width = self.resources.font.string_width(&self.name_buffer) as i32;
                    if self.name_buffer.len() < 20 && width < 110 {
                        self.name_buffer.push(c);
                    }
                }
                Key::Backspace => {
                    self.name_buffer.pop();
                }
                Key::Enter => {
                    self.finalize_current_name();
                }
                _ => {}
            }
            return None;
        }

        if self.phase == ViewPhase::Ready {
            if matches!(event, Event::Keyboard(_)) {
                self.phase = ViewPhase::ShowTeams;
            }
            return None;
        }

        if self.phase == ViewPhase::ShowTeams {
            if matches!(event, Event::Keyboard(_)) {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible();
            }
            return None;
        }

        if let Some(ref scene) = self.scene {
            if handle_save_dialog(scene, &event) {
                return None;
            }

            // Let the shared input controller process events first
            if self.phase == ViewPhase::Jumping {
                match handle_competition_jump_input(scene, event, false) {
                    JumpInputResult::Route(route) => return Some(route),
                    JumpInputResult::Consumed => return None,
                    JumpInputResult::None => {}
                }
            }

            if !self.ui_state.is_outcome_recorded()
                && acknowledge_finished_jump(scene, &self.ui_state, event, false)
            {
                return None;
            }
        }

        if self.ui_state.render_mode() == RenderMode::Results {
            if matches!(event, Event::Keyboard(_)) {
                self.ui_state.dismiss_results();
                if let Some(ref scene) = self.scene {
                    match self.session.advance_results_and_drive::<TeamCupRuntime>(
                        scene,
                        TeamCupResultsKind::LegResults,
                    ) {
                        Ok(Some(cmd)) => self.apply_command(cmd),
                        Ok(None) => {}
                        Err(e) => self.ui_state.enter_error(e.to_string()),
                    }
                }
            }
            return None;
        }

        None
    }
}
