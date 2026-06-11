use super::results;
use crate::competition::machine::Competition;
use crate::competition::runtime::{
    IndividualJumpContext, IndividualResultsKind,
};
use crate::competition::scoring::wc_points_for_rank;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::components::screen;
use crate::gfx::palette::FONT_GREET;
use crate::jump::types::JumpPhase;
use crate::jump::JumpParticipant;
use crate::jump::JumpPolicy;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_decimal;
use crate::views::jump::competition::flow::{
    acknowledge_finished_jump, handle_save_dialog, record_acknowledged_human_jump,
    handle_competition_jump_input, render_jump_scene_with_overlay, route_error_back,
    CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::session::CompetitionSession;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode, ResultScreen};
use crate::views::jump::scene::JumpScene;
use engine::ui::{Blinker, Element, Event, Key, View};

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    blinker: Blinker,
    session: CompetitionSession,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
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
            session: CompetitionSession::new(resources, store),
        }
    }

    fn apply_command(
        &self,
        command: CompetitionFlowCommand<IndividualJumpContext, IndividualResultsKind>,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event,
            } => {
                if is_new_event {
                    self.store.setup_jump_event();
                }
                let phase_label = phase_label(&self.resources, context.phase);
                self.handle_human_jump(participant, hill_idx, phase_label);
                self.ui_state.enter_jump();
            }
            CompetitionFlowCommand::ShowResults(IndividualResultsKind::Results) => {
                if self.ui_state.render_mode() != RenderMode::Results {
                    self.select_default_result_screen();
                    self.ui_state.enter_results();
                }
            }
            CompetitionFlowCommand::Done => {
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
        if let Some(phase) = self
            .store
            .with_active(|active| active.individual().map(Competition::phase))
            .flatten()
        {
            let is_4h = self
                .store
                .with_active(|active| active.individual().map(Competition::is_four_hills_event))
                .flatten()
                .unwrap_or(false);
            self.ui_state.select_default_screen(is_4h, phase);
        }
    }

    fn results_page(&self) -> Vec<Element> {
        self.store
            .with_active(|active| {
                let c = active.individual()?;
                Some(match self.ui_state.current_screen() {
                    ResultScreen::KoPairs(show_results) => {
                        let show_cursor = self.blinker.visible(10, 10);
                        results::render_ko_pairs(
                            c,
                            &self.resources,
                            show_results,
                            show_cursor,
                        )
                    }
                    ResultScreen::Stats => results::render_stats_page(
                            c,
                            &self.resources,
                            self.ui_state.current_page(),
                        ),
                    ResultScreen::List => {
                        let page_data = if self.ui_state.is_compact() {
                            results::build_compact_results_page(c)
                        } else {
                            results::build_results_page(c, self.ui_state.current_page())
                        };
                        let mut els = results::render_results_page(&page_data, &self.resources);
                        els.extend(results::render_header(c, &self.resources));
                        els
                    }
                })
            })
            .flatten()
            .unwrap_or_else(screen::black_screen)
    }

    /// Pascal: rank calculation — counts participants with points <= jumper's total.
    /// Shows `($X.)` at (255,45), left of the score at (308,45).
    fn rank_element(&self) -> Option<Element> {
        let outcome = self.scene.outcome()?;
        if self.scene.phase() != Some(JumpPhase::Result) {
            return None;
        }
        let own_id = self.scene.participant_id();
        self.store.with_active(|active| {
            let c = active.individual()?;
            let standings = c.event_standings();
            let own_before = standings
                .iter()
                .find(|p| p.id == own_id)
                .and_then(|p| p.points)
                .unwrap_or(0.0);
            let own_total = own_before + outcome.score;
            let rank = standings
                .iter()
                .filter(|p| p.id != own_id && p.points.is_some_and(|pts| pts > own_total))
                .count()
                + 1;
            Some(Element::right_text(format!("(${rank}.)"), 255, 45, FONT_GREET))
        })
        .flatten()
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        record_acknowledged_human_jump::<Competition>(
            &self.session,
            &self.ui_state,
            Some(&self.scene),
        );

        // Drive competition and dispatch any resulting command
        match self.session.drive_competition::<Competition>(&self.scene) {
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
                let mut els = render_jump_scene_with_overlay(&self.scene, &self.overlay, &self.ui_state);
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
                screen::message_screen(&msg, "Press any key to return")
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        // Error screen: any key navigates back to main menu
        if let Some(route) = route_error_back(&self.ui_state, event) {
            return Some(route);
        }
        if self.ui_state.render_mode() == RenderMode::Error {
            return None;
        }

        if self.is_result_display_state() {
            return self.handle_result_event(event);
        }

        // Replay save dialog
        if handle_save_dialog(&self.scene, &event) {
            return None;
        }

        // Let the shared input controller process events first (save replay, etc.)
        match handle_competition_jump_input(&self.scene, event, true) {
            JumpInputResult::Route(route) => return Some(route),
            JumpInputResult::Consumed => return None,
            JumpInputResult::None => {}
        }

        // Pascal: wait for key after human jump before advancing
        let is_dq = self.scene.phase() == Some(JumpPhase::Disqualified);
        if acknowledge_finished_jump(&self.scene, &self.ui_state, event, !is_dq) {
            return None;
        }

        None
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.ui_state.has_page() {
            return true;
        }
        self.store
            .with_active(|active| {
                let c = active.individual()?;
                Some(
                c.phase().is_result_phase()
                    || c.phase().needs_event_results() && c.current_jumper().is_none()
                )
            })
            .flatten()
            .unwrap_or(false)
    }

    fn save_competition_results(&self) {
        self.session.save_results();
        // WC-specific profile updates (bestpoints, etc.)
        self.store.with_active(|active| {
            let Some(c) = active.individual() else {
                return;
            };
            let style = c.style();
            let overall = c.overall_standings();
            let mut profiles = self.store.profiles_mut();
            for p in &overall {
                let Some(pidx) = p.profile_idx else { continue };
                let Some(profile) = profiles.profiles.get_mut(pidx) else {
                    continue;
                };
                match style {
                    CupStyle::WorldCup => {
                        profile.world_cups += 1;
                        if p.points.is_none() {
                            continue;
                        }
                        let my_points = p.points.unwrap();
                        let event_rank = event_rank_by_points(c, my_points);
                        let pts = wc_points_for_rank(event_rank);
                        if pts >= profile.bestpoints as i32 {
                            profile.bestpoints = pts as usize;
                            profile.best_result = format_wc_best_result(pts, event_rank);
                        }
                        if p.four_hills_points > 0.0 && p.four_hills_points >= profile.best4points {
                            profile.best4points = p.four_hills_points;
                            profile.best_4h_result =
                                format_four_hills_best_result(p.four_hills_points, p.rank);
                        }
                    }
                    CupStyle::FourHills => {
                        if p.four_hills_points >= profile.best4points {
                            profile.best4points = p.four_hills_points;
                            profile.best_4h_result =
                                format_four_hills_best_result(p.four_hills_points, p.rank);
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn handle_result_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                let total = match self.ui_state.current_screen() {
                    ResultScreen::Stats => self
                        .store
                        .with_active(|active| {
                            let c = active.individual()?;
                            Some(
                            c.overall_standings()
                                .iter()
                                .filter(|p| !p.is_computer)
                                .count()
                                .max(1)
                            )
                        })
                        .flatten()
                        .unwrap_or(1),
                    ResultScreen::KoPairs(_) => 1,
                    ResultScreen::List if self.ui_state.is_compact() => 1,
                    ResultScreen::List => self
                        .store
                        .with_active(|active| active.individual().map(results::total_pages))
                        .flatten()
                        .unwrap_or(0),
                };
                if self.ui_state.next_page(total) {
                    return None;
                }
                // Pascal WaitForKey(0): any key on the last entry exits the list
                self.blinker.reset();
                self.ui_state.dismiss_results();
                match self.session.advance_results_and_drive::<Competition>(
                    &self.scene,
                    IndividualResultsKind::Results,
                ) {
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
                    .with_active(|active| {
                        let c = active.individual()?;
                        Some(
                        c.is_four_hills_event()
                            && matches!(
                                c.phase(),
                                CompetitionPhase::QualificationResults
                                    | CompetitionPhase::Round1Results
                            )
                        )
                    })
                    .flatten()
                    .unwrap_or(false);
                if ko {
                    let round1 = self
                        .store
                        .with_active(|active| {
                            active
                                .individual()
                                .map(|c| c.phase() == CompetitionPhase::Round1Results)
                        })
                        .flatten()
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
                    .with_active(|active| {
                        active
                            .individual()
                            .map(|c| c.phase() == CompetitionPhase::SeasonComplete)
                    })
                    .flatten()
                    .unwrap_or(false);
                if is_season_complete {
                    self.save_competition_results();
                    return Some(RouteTarget::Back);
                }
                self.blinker.reset();
                self.ui_state.dismiss_results();
                match self.session.advance_results_and_drive::<Competition>(
                    &self.scene,
                    IndividualResultsKind::Results,
                ) {
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

fn format_wc_best_result(points: i32, rank: usize) -> String {
    format!("{points} ({rank}.)")
}

fn format_four_hills_best_result(points: f64, rank: usize) -> String {
    format!("{} ({}.)", format_decimal(points), rank)
}

/// Tie-aware event rank: 1 + count of participants with strictly higher points.
fn event_rank_by_points(competition: &Competition, points: f64) -> usize {
    1 + competition
        .event_standings()
        .iter()
        .filter(|p| p.points.unwrap_or(f64::NEG_INFINITY) > points)
        .count()
}
