use super::results;
use crate::competition::machine::Competition;
use crate::competition::runtime::{IndividualJumpContext, IndividualResultsKind};
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
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::results::{
    self as competition_results, CompetitionResultsRequest,
};
use crate::views::jump::competition::ui_state::{RenderMode, ResultScreen};
use crate::views::jump::scene::JumpScene;
use engine::oxide::legacy::{commands_to_elements, event_from_ui, paint_elements};
use engine::oxide::{CommandBuffer, NavAction, PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Blinker, Element, Event, Key, View};

pub struct WorldCupJumpView {
    controller: CompetitionJumpController<Competition>,
    blinker: Blinker,
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
            controller: CompetitionJumpController::new(resources, store, Some(scene)),
            blinker: Blinker::new(),
        }
    }

    fn apply_command(
        &mut self,
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
                    self.controller.store().setup_jump_event();
                }
                let phase_label = phase_label(self.controller.resources(), context.phase);
                self.controller
                    .prepare_human_jump(participant, hill_idx, phase_label, None);
            }
            CompetitionFlowCommand::ShowResults(IndividualResultsKind::Results) => {
                if self.controller.render_mode() != RenderMode::Results {
                    self.select_default_result_screen();
                    self.controller.enter_results();
                }
            }
            CompetitionFlowCommand::Done => {
                self.controller.enter_done();
            }
        }
    }

    fn select_default_result_screen(&self) {
        if let Some(phase) = self
            .controller
            .store()
            .with_active(|active| active.individual().map(Competition::phase))
            .flatten()
        {
            let is_4h = self
                .controller
                .store()
                .with_active(|active| active.individual().map(Competition::is_four_hills_event))
                .flatten()
                .unwrap_or(false);
            self.controller
                .ui_state()
                .select_default_screen(is_4h, phase);
        }
    }

    fn results_page(&self) -> Vec<Element> {
        competition_results::render(
            self.controller.resources(),
            self.controller.store(),
            self.controller.ui_state(),
            CompetitionResultsRequest::Individual {
                ko_cursor_visible: self.blinker.visible(10, 10),
            },
        )
    }

    /// Pascal: rank calculation — counts participants with points <= jumper's total.
    /// Shows `($X.)` at (255,45), left of the score at (308,45).
    fn rank_element(&self) -> Option<Element> {
        let scene = self.controller.scene()?;
        let outcome = scene.outcome()?;
        if scene.phase() != Some(JumpPhase::Result) {
            return None;
        }
        let own_id = scene.participant_id();
        self.controller
            .store()
            .with_active(|active| {
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
                Some(Element::right_text(
                    format!("(${rank}.)"),
                    255,
                    45,
                    FONT_GREET,
                ))
            })
            .flatten()
    }

    fn legacy_elements(&self) -> Vec<Element> {
        match self.controller.render_mode() {
            RenderMode::Jump => {
                let mut els = self.controller.render_jump_elements();
                // Pascal: show rank ($X.) left of score at (255,45) during Result phase
                if let Some(rank_el) = self.rank_element() {
                    els.push(rank_el);
                }
                els
            }
            RenderMode::Results => self.results_page(),
            RenderMode::Done => vec![],
            RenderMode::Error => {
                let msg = self.controller.ui_state().error_message();
                screen::message_screen(&msg, "Press any key to return")
            }
        }
    }

    fn handle_legacy_event(&mut self, event: Event) -> Option<RouteTarget> {
        // Error screen: any key navigates back to main menu
        if let Some(route) = route_error_back(self.controller.ui_state(), event) {
            return Some(route);
        }
        if self.controller.render_mode() == RenderMode::Error {
            return None;
        }

        if self.is_result_display_state() {
            return self.handle_result_event(event);
        }

        // Let the shared input controller process events first (save replay, etc.)
        let is_dq =
            self.controller.scene().and_then(JumpScene::phase) == Some(JumpPhase::Disqualified);
        match self
            .controller
            .handle_jump_scene_event(event, true, !is_dq, false)
        {
            JumpInputResult::Route(route) => return Some(route),
            JumpInputResult::Consumed => return None,
            JumpInputResult::None => {}
        }

        None
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        self.controller.record_acknowledged_human_jump();

        // Drive competition and dispatch any resulting command
        if let Some(command) = self.controller.drive() {
            self.apply_command(command);
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene();
        }
    }

    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
        }
    }
}

impl Screen<RouteTarget> for WorldCupJumpView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
            return;
        };
        if let Some(route) = self.handle_legacy_event(event) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        paint_elements(cx, &self.legacy_elements());
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.controller.ui_state().has_page() {
            return true;
        }
        self.controller
            .store()
            .with_active(|active| {
                let c = active.individual()?;
                Some(
                    c.phase().is_result_phase()
                        || c.phase().needs_event_results() && c.current_jumper().is_none(),
                )
            })
            .flatten()
            .unwrap_or(false)
    }

    fn save_competition_results(&self) {
        self.controller.session().save_results();
        // WC-specific profile updates (bestpoints, etc.)
        self.controller.store().with_active(|active| {
            let Some(c) = active.individual() else {
                return;
            };
            let style = c.style();
            let overall = c.overall_standings();
            let mut profiles = self.controller.store().profiles_mut();
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

    fn dismiss_results_and_advance(&mut self) {
        self.blinker.reset();
        if let Some(command) = self
            .controller
            .dismiss_results_and_advance(IndividualResultsKind::Results)
        {
            self.apply_command(command);
        }
    }

    fn handle_result_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                let total = match self.controller.ui_state().current_screen() {
                    ResultScreen::Stats => self
                        .controller
                        .store()
                        .with_active(|active| {
                            let c = active.individual()?;
                            Some(
                                c.overall_standings()
                                    .iter()
                                    .filter(|p| !p.is_computer)
                                    .count()
                                    .max(1),
                            )
                        })
                        .flatten()
                        .unwrap_or(1),
                    ResultScreen::KoPairs(_) => 1,
                    ResultScreen::List if self.controller.ui_state().is_compact() => 1,
                    ResultScreen::List => self
                        .controller
                        .store()
                        .with_active(|active| active.individual().map(results::total_pages))
                        .flatten()
                        .unwrap_or(0),
                };
                if self.controller.ui_state().next_page(total) {
                    return None;
                }
                // Pascal WaitForKey(0): any key on the last entry exits the list
                self.dismiss_results_and_advance();
                None
            }
            Event::Keyboard(Key::Char('c' | 'C')) => {
                self.controller.ui_state().toggle_compact();
                None
            }
            Event::Keyboard(Key::Char('s' | 'S')) => {
                self.controller.ui_state().toggle_stats();
                None
            }
            Event::Keyboard(Key::Char('k' | 'K')) => {
                let ko = self
                    .controller
                    .store()
                    .with_active(|active| {
                        let c = active.individual()?;
                        Some(
                            c.is_four_hills_event()
                                && matches!(
                                    c.phase(),
                                    CompetitionPhase::QualificationResults
                                        | CompetitionPhase::Round1Results
                                ),
                        )
                    })
                    .flatten()
                    .unwrap_or(false);
                if ko {
                    let round1 = self
                        .controller
                        .store()
                        .with_active(|active| {
                            active
                                .individual()
                                .map(|c| c.phase() == CompetitionPhase::Round1Results)
                        })
                        .flatten()
                        .unwrap_or(false);
                    self.controller.ui_state().toggle_ko_pairs(round1);
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                self.controller.ui_state().prev_page();
                None
            }
            Event::Keyboard(Key::Escape | Key::Enter) => {
                let is_season_complete = self
                    .controller
                    .store()
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
                self.dismiss_results_and_advance();
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
