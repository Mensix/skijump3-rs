use super::overlay::CompetitionOverlay;
use super::results;
use super::session::{WorldCupSessionController, WorldCupUiCommand};
use super::ui_state::{CompetitionUiState, RenderMode, ResultScreen};
use crate::competition::machine::Competition;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::gfx::palette::{BLACK, FONT_GREET};
use crate::jump::types::JumpPhase;
use crate::jump::JumpParticipant;
use crate::jump::JumpPolicy;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::input::{JumpInputAction, JumpInputController};
use crate::views::jump::scene::JumpScene;
use engine::ui::{Blinker, Element, Event, Key, View};

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    blinker: Blinker,
    controller: WorldCupSessionController,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        JumpScene::setup_event(&store);
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            StoreRef::clone(&store),
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        );
        Self {
            resources: ResourcesRef::clone(&resources),
            store: StoreRef::clone(&store),
            scene,
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(
                ResourcesRef::clone(&resources),
                StoreRef::clone(&store),
            ),
            blinker: Blinker::new(),
            controller: WorldCupSessionController::new(resources, store),
        }
    }

    fn apply_command(&self, command: WorldCupUiCommand) {
        match command {
            WorldCupUiCommand::HumanJump(req) => {
                if req.is_new_event {
                    JumpScene::setup_event(&self.store);
                }
                let phase_label = phase_label(&self.resources, req.phase);
                self.handle_human_jump(req.participant, req.hill_idx, phase_label);
                self.ui_state.enter_jump();
            }
            WorldCupUiCommand::ShowResults => {
                if self.ui_state.render_mode() != RenderMode::Results {
                    self.select_default_result_screen();
                    self.ui_state.enter_results();
                }
            }
            WorldCupUiCommand::Done => {
                self.ui_state.enter_done();
            }
        }
    }

    fn handle_human_jump(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = self.scene.participant_id() != participant.id
            || self.scene.hill_idx() != hill_idx
            || self.ui_state.is_outcome_recorded()
            || (self.scene.outcome().is_some() && self.ui_state.is_result_acknowledged());
        if needs_rebuild {
            self.ui_state.reset_acknowledged();
            self.ui_state.reset_outcome_recorded();
            self.scene
                .rebuild_for_competition(hill_idx, 15, participant, phase_label);
        } else {
            self.scene.set_phase_label(phase_label);
        }
    }

    fn select_default_result_screen(&self) {
        if let Some(phase) = self.store.try_with_competition(Competition::phase) {
            let is_4h = self
                .store
                .try_with_competition(Competition::is_four_hills_event)
                .unwrap_or(false);
            self.ui_state.select_default_screen(is_4h, phase);
        }
    }

    fn results_page(&self) -> Vec<Element> {
        self.store
            .try_with_competition(|c| {
                match self.ui_state.current_screen() {
                    ResultScreen::KoPairs(show_results) => {
                        let show_cursor = self.blinker.visible(10, 10);
                        return results::render_ko_pairs(
                            c,
                            &self.resources,
                            show_results,
                            show_cursor,
                        );
                    }
                    ResultScreen::Stats => {
                        return results::render_stats_page(
                            c,
                            &self.resources,
                            self.ui_state.current_page(),
                        );
                    }
                    ResultScreen::List => {}
                }
                let page_data = if self.ui_state.is_compact() {
                    results::build_compact_results_page(c)
                } else {
                    results::build_results_page(c, self.ui_state.current_page())
                };
                let mut els = results::render_results_page(&page_data, &self.resources);
                els.extend(results::render_header(c, &self.resources));
                els
            })
            .unwrap_or_else(|| vec![Element::fillbox(0, 0, 320, 200, BLACK)])
    }

    /// Pascal: rank calculation — counts participants with points <= jumper's total.
    /// Shows `($X.)` at (255,45), left of the score at (308,45).
    fn rank_element(&self) -> Option<Element> {
        let outcome = self.scene.outcome()?;
        if self.scene.phase() != Some(JumpPhase::Result) {
            return None;
        }
        let own_id = self.scene.participant_id();
        self.store.try_with_competition(|c| {
            let standings = c.event_standings();
            let own_before = standings
                .iter()
                .find(|p| p.id == own_id)
                .and_then(|p| p.points)
                .unwrap_or(0);
            let own_total = own_before + outcome.score;
            let rank = standings
                .iter()
                .filter(|p| p.id != own_id && p.points.is_some_and(|pts| pts > own_total))
                .count()
                + 1;
            Element::right_text(format!("(${rank}.)"), 255, 45, FONT_GREET)
        })
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        // Record acknowledged human jump outcome if not yet recorded
        if self.ui_state.is_result_acknowledged()
            && !self.ui_state.is_outcome_recorded()
            && self.controller.record_finished_human_jump(&self.scene)
        {
            self.ui_state.mark_outcome_recorded();
        }

        // Drive competition and dispatch any resulting command
        match self.controller.drive_competition(&self.scene) {
            Ok(Some(command)) => self.apply_command(command),
            Ok(None) => {}
            Err(e) => self.ui_state.enter_error(e.to_string()),
        }

        if self.ui_state.render_mode() == RenderMode::Jump {
            self.scene.update();
        }
    }

    fn elements(&self) -> Vec<Element> {
        match self.ui_state.render_mode() {
            RenderMode::Jump => {
                // Suppress static InfoPanel text when overlays provide their own content:
                // Round 2 cycling info, or the keymap for the first human's first event.
                let hide = self
                    .store
                    .try_with_competition(|c| {
                        let cycling =
                            c.phase().needs_event_results() && c.style() != CupStyle::CustomCup;
                        let keymap_active = self.ui_state.is_first_human_onbar()
                            && c.current_event == 0
                            && c.current_jumper()
                                .is_some_and(|idx| !c.participant(idx).is_computer);
                        cycling || keymap_active
                    })
                    .unwrap_or(false);
                self.scene.set_hide_info_panel_text(hide);
                let mut els = self.scene.elements();
                // Overlay: keymap / cycling info / jumper info box
                if let Some(ctx) = self.overlay.context(
                    self.scene.phase(),
                    self.scene.frame_counter(),
                    &self.ui_state,
                ) {
                    els.extend(self.overlay.render_elements(&ctx));
                }
                // Pascal: show rank ($X.) left of score at (255,45) during Result phase
                if let Some(rank_el) = self.rank_element() {
                    els.push(rank_el);
                }
                els
            }
            RenderMode::Results => self.results_page(),
            RenderMode::Done => vec![],
            RenderMode::Error => {
                let msg = self.ui_state.error_message();
                vec![
                    Element::fillbox(0, 0, 320, 200, BLACK),
                    Element::text(&msg, 10, 10, FONT_GREET, false),
                    Element::text("Press any key to return", 10, 180, FONT_GREET, false),
                ]
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        // Error screen: any key navigates back to main menu
        if self.ui_state.render_mode() == RenderMode::Error {
            if matches!(event, Event::Keyboard(_)) {
                return Some(RouteTarget::Back);
            }
            return None;
        }

        if self.is_result_display_state() {
            return self.handle_result_event(event);
        }

        // Pascal: wait for key after human jump before advancing
        if self.scene.outcome().is_some() && !self.ui_state.is_result_acknowledged() {
            let is_dq = self.scene.phase() == Some(JumpPhase::Disqualified);
            let accepted = if is_dq {
                // Pascal waitforkey: ANY key dismisses the DQ screen
                matches!(event, Event::Keyboard(_))
            } else {
                matches!(event, Event::Keyboard(Key::Enter | Key::Escape))
            };
            if accepted {
                self.ui_state.acknowledge_outcome();
                return None;
            }
            return None;
        }

        let action = {
            let mut session = self.scene.session_mut();
            JumpInputController.handle_event(event, &mut session)
        };
        match action {
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            _ => None,
        }
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.ui_state.has_page() {
            return true;
        }
        self.store
            .try_with_competition(|c| {
                c.phase().is_result_phase()
                    || c.phase().needs_event_results() && c.current_jumper().is_none()
            })
            .unwrap_or(false)
    }

    fn handle_result_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                let total = match self.ui_state.current_screen() {
                    ResultScreen::Stats => self
                        .store
                        .try_with_competition(|c| {
                            c.overall_standings()
                                .iter()
                                .filter(|p| !p.is_computer)
                                .count()
                                .max(1)
                        })
                        .unwrap_or(1),
                    ResultScreen::KoPairs(_) => 1,
                    ResultScreen::List if self.ui_state.is_compact() => 1,
                    ResultScreen::List => self
                        .store
                        .try_with_competition(results::total_pages)
                        .unwrap_or(0),
                };
                if self.ui_state.next_page(total) {
                    return None;
                }
                // Pascal WaitForKey(0): any key on the last entry exits the list
                self.blinker.reset();
                self.ui_state.dismiss_results();
                self.store.try_with_competition_mut(Competition::advance);
                match self.controller.drive_competition(&self.scene) {
                    Ok(Some(command)) => self.apply_command(command),
                    Ok(None) => {}
                    Err(e) => self.ui_state.enter_error(e.to_string()),
                }
                None
            }
            Event::Keyboard(Key::Char('c' | 'C')) => {
                self.ui_state.toggle_compact();
                None
            }
            Event::Keyboard(Key::Char('s' | 'S')) => {
                self.ui_state.toggle_stats();
                None
            }
            Event::Keyboard(Key::Char('k' | 'K')) => {
                let ko = self
                    .store
                    .try_with_competition(|c| {
                        c.is_four_hills_event()
                            && matches!(
                                c.phase(),
                                CompetitionPhase::QualificationResults
                                    | CompetitionPhase::Round1Results
                            )
                    })
                    .unwrap_or(false);
                if ko {
                    let round1 = self
                        .store
                        .try_with_competition(|c| c.phase() == CompetitionPhase::Round1Results)
                        .unwrap_or(false);
                    self.ui_state.toggle_ko_pairs(round1);
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                self.ui_state.prev_page();
                None
            }
            Event::Keyboard(Key::Escape | Key::Enter) => {
                let is_season_complete = self
                    .store
                    .try_with_competition(|c| c.phase() == CompetitionPhase::SeasonComplete)
                    .unwrap_or(false);
                if is_season_complete {
                    self.controller.save_competition_results();
                    return Some(RouteTarget::Back);
                }
                self.blinker.reset();
                self.ui_state.dismiss_results();
                self.store.try_with_competition_mut(Competition::advance);
                match self.controller.drive_competition(&self.scene) {
                    Ok(Some(command)) => self.apply_command(command),
                    Ok(None) => {}
                    Err(e) => self.ui_state.enter_error(e.to_string()),
                }
                None
            }
            _ => None,
        }
    }
}

fn phase_label(resources: &ResourcesRef, phase: CompetitionPhase) -> String {
    match phase {
        CompetitionPhase::Training(n) => format!("{} {}", resources.langbase.lstr(52), n),
        CompetitionPhase::Qualification => resources.langbase.lstr(53).to_string(),
        CompetitionPhase::Round1 => resources.langbase.lstr(54).to_string(),
        CompetitionPhase::Round2 => resources.langbase.lstr(55).to_string(),
        _ => resources.langbase.lstr(51).to_string(),
    }
}
