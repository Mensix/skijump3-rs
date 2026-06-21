use super::results;
use crate::competition::machine::Competition;
use crate::competition::runtime::{IndividualJumpContext, IndividualResultsKind};
use crate::competition::scoring::wc_points_for_rank;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::gfx::theme::FONT_TEAL;
use crate::jump::types::JumpPhase;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::current_timestamp;
use crate::text::format::format_decimal;
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::results::{
    self as competition_results, CompetitionResultsRequest,
};
use crate::views::jump::competition::ui_state::{RenderMode, ResultScreen};
use engine::oxide::Blinker;
use engine::oxide::{Key, PaintCx, ScreenEventCx, UiEvent};

pub struct WorldCupJumpView {
    controller: CompetitionJumpController<Competition>,
    blinker: Blinker,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, save_manager: SaveRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, save_manager, false, None),
            blinker: Blinker::new(),
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<IndividualJumpContext, IndividualResultsKind>,
        state: &mut GameState,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event,
            } => {
                if is_new_event {
                    state.setup_jump_event();
                }
                let phase_label = phase_label(self.controller.resources(), context.phase);
                let is_leader = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| {
                        active.individual().and_then(|c| {
                            c.overall_standings()
                                .first()
                                .map(|s| s.id == participant.id)
                        })
                    })
                    .unwrap_or(false);
                self.controller
                    .prepare_human_jump(participant, hill_idx, phase_label, None, state);
                if let Some(scene) = self.controller.scene_mut() {
                    scene.set_has_bib(is_leader);
                }
            }
            CompetitionFlowCommand::ShowResults(IndividualResultsKind::Results) => {
                if self.controller.render_mode() != RenderMode::Results {
                    self.select_default_result_screen(state);
                    self.controller.enter_results();
                }
            }
            CompetitionFlowCommand::Done => {
                self.controller.enter_done();
            }
        }
    }

    fn select_default_result_screen(&mut self, state: &GameState) {
        if let Some(phase) = state
            .active_competition
            .as_ref()
            .and_then(|active| active.individual().map(Competition::phase))
        {
            let is_4h = state
                .active_competition
                .as_ref()
                .and_then(|active| active.individual().map(Competition::is_four_hills_event))
                .unwrap_or(false);
            self.controller
                .ui_state_mut()
                .select_default_screen(is_4h, phase);
            let extra_stats_enabled = state.config.extra_statistics != 0;
            let showing_ko_pairs = is_4h
                && matches!(
                    phase,
                    CompetitionPhase::QualificationResults | CompetitionPhase::Round1Results
                );
            if extra_stats_enabled && !showing_ko_pairs {
                self.controller.ui_state_mut().select_stats_screen();
            }
        }
    }

    fn results_page(&self, cx: &mut PaintCx<'_>, state: &GameState) {
        competition_results::render(
            cx,
            self.controller.resources(),
            state,
            self.controller.ui_state(),
            CompetitionResultsRequest::Individual {
                ko_cursor_visible: self.blinker.visible(10, 10),
            },
        )
    }

    fn draw_rank(&self, cx: &mut PaintCx<'_>, state: &GameState) {
        let Some(scene) = self.controller.scene() else {
            return;
        };
        let Some(outcome) = scene.outcome() else {
            return;
        };
        if scene.phase() != Some(JumpPhase::Result) {
            return;
        }
        let own_id = scene.participant_id();
        if let Some(rank) = state.active_competition.as_ref().and_then(|active| {
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
            Some(rank)
        }) {
            cx.right_text((255, 45), FONT_TEAL, format!("(${rank}.)"));
        }
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>, state: &GameState) {
        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
                self.draw_rank(cx, state);
            }
            RenderMode::Results => self.results_page(cx, state),
            RenderMode::Done => {}
        }
    }

    fn handle_input(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        if self.is_result_display_state(state) {
            return self.handle_result_event(event, state);
        }

        let is_dq =
            self.controller.scene().and_then(|s| s.phase()) == Some(JumpPhase::Disqualified);
        match self
            .controller
            .handle_jump_scene_event(event, true, !is_dq, false, state)
        {
            JumpInputResult::Consumed => return None,
            JumpInputResult::None => {}
        }

        None
    }
}

impl GameScreen for WorldCupJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.controller.record_acknowledged_human_jump(cx.state);

        if let Some(command) = self.controller.drive(cx.state) {
            self.apply_command(command, cx.state);
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene(cx.state);
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event, cx.state) {
            if route == RouteTarget::Back {
                nav.back();
            } else {
                nav.navigate(route);
            }
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(paint, cx.state);
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self, state: &GameState) -> bool {
        if self.controller.ui_state().has_page() {
            return true;
        }
        state
            .active_competition
            .as_ref()
            .and_then(|active| {
                let c = active.individual()?;
                Some(
                    c.phase().is_result_phase()
                        || c.phase().needs_event_results() && c.current_jumper().is_none(),
                )
            })
            .unwrap_or(false)
    }

    fn save_competition_results(&mut self, state: &mut GameState) {
        self.update_hall_of_fame(state);
        self.controller.save_results(state);

        let (style, participants, event_pts): (_, Vec<_>, Vec<_>) = {
            let active = match state.active_competition.as_ref() {
                Some(a) => a,
                None => return,
            };
            let Some(c) = active.individual() else {
                return;
            };
            let style = c.style();
            let participants = c
                .overall_standings()
                .into_iter()
                .map(|p| {
                    (
                        p.profile_idx,
                        p.points,
                        p.four_hills_points,
                        p.rank,
                        p.leg_wins,
                    )
                })
                .collect();
            let event_pts = c.event_standings().into_iter().map(|p| p.points).collect();
            (style, participants, event_pts)
        };
        let profiles = &mut state.profiles;
        for (pos, &(pidx_opt, pts_opt, fh_points, rank, leg_wins)) in participants.iter().enumerate() {
            let Some(pidx) = pidx_opt else { continue };
            let Some(profile) = profiles.profiles.get_mut(pidx) else {
                continue;
            };
            match style {
                CupStyle::WorldCup => {
                    profile.world_cups += 1;
                    profile.legs_won += leg_wins;
                    if pos == 0 {
                        profile.world_cups_won += 1;
                    }
                    let Some(my_points) = pts_opt else { continue };
                    let event_rank = 1 + event_pts
                        .iter()
                        .filter(|&&sp_pts| sp_pts.unwrap_or(f64::NEG_INFINITY) > my_points)
                        .count();
                    let pts = wc_points_for_rank(event_rank);
                    if pts >= profile.best_points as i32 {
                        profile.best_points = pts as usize;
                        profile.best_result = format_wc_best_result(pts, event_rank);
                    }
                    if fh_points > 0.0 && fh_points >= profile.best_4h_points {
                        profile.best_4h_points = fh_points;
                        profile.best_4h_result = format_four_hills_best_result(fh_points, rank);
                    }
                }
                CupStyle::FourHills if fh_points >= profile.best_4h_points => {
                    profile.best_4h_points = fh_points;
                    profile.best_4h_result = format_four_hills_best_result(fh_points, rank);
                }
                _ => {}
            }
        }
    }

    fn update_hall_of_fame(&self, state: &mut GameState) {
        let (style, wc_top, fh_top) = {
            let Some(active) = state.active_competition.as_ref() else {
                return;
            };
            let Some(c) = active.individual() else { return };
            let participants = c.overall_standings();
            let style = c.style();

            let wc_top: Vec<_> = participants
                .iter()
                .take(20)
                .map(|p| {
                    (
                        p.display_name().to_string(),
                        p.wc_points as f64,
                        p.is_computer,
                    )
                })
                .collect();

            let mut fh: Vec<_> = participants
                .iter()
                .filter(|p| p.four_hills_points > 0.0)
                .map(|p| {
                    (
                        p.display_name().to_string(),
                        p.four_hills_points,
                        p.is_computer,
                    )
                })
                .collect();
            fh.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            (style, wc_top, fh)
        };

        match style {
            CupStyle::WorldCup => {
                for (i, (name, score, is_computer)) in wc_top.iter().enumerate() {
                    let slot = i;
                    if let Some(record) = state.records.top.get_mut(slot) {
                        *record = crate::data::records::Hiscore {
                            name: name.clone(),
                            pos: i + 1,
                            score: *score,
                            time: current_timestamp(),
                            is_computer: *is_computer,
                        };
                    }
                }
                for (i, (name, score, is_computer)) in fh_top.iter().enumerate().take(5) {
                    let slot = 30 + i;
                    if let Some(record) = state.records.top.get_mut(slot) {
                        *record = crate::data::records::Hiscore {
                            name: name.clone(),
                            pos: i + 1,
                            score: *score,
                            time: current_timestamp(),
                            is_computer: *is_computer,
                        };
                    }
                }
            }
            CupStyle::FourHills => {
                for (i, (name, score, is_computer)) in fh_top.iter().enumerate().take(5) {
                    let slot = 30 + i;
                    if let Some(record) = state.records.top.get_mut(slot) {
                        *record = crate::data::records::Hiscore {
                            name: name.clone(),
                            pos: i + 1,
                            score: *score,
                            time: current_timestamp(),
                            is_computer: *is_computer,
                        };
                    }
                }
            }
            _ => {}
        }
    }

    fn dismiss_results_and_advance(&mut self, state: &mut GameState) {
        self.blinker.reset();
        if let Some(command) = self.controller.dismiss_results_and_advance(state) {
            self.apply_command(command, state);
        }
    }

    fn handle_result_event(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
    ) -> Option<RouteTarget> {
        match event {
            UiEvent::KeyDown(Key::Right) | UiEvent::Text(' ') => {
                let total = match self.controller.ui_state().current_screen() {
                    ResultScreen::Stats => state
                        .active_competition
                        .as_ref()
                        .and_then(|active| {
                            let c = active.individual()?;
                            Some(
                                c.overall_standings()
                                    .iter()
                                    .filter(|p| !p.is_computer)
                                    .count()
                                    .max(1),
                            )
                        })
                        .unwrap_or(1),
                    ResultScreen::KoPairs(_) => 1,
                    ResultScreen::List if self.controller.ui_state().is_compact() => 1,
                    ResultScreen::List => state
                        .active_competition
                        .as_ref()
                        .and_then(|active| active.individual().map(results::total_pages))
                        .unwrap_or(0),
                };
                if self.controller.ui_state_mut().next_page(total) {
                    return None;
                }

                self.dismiss_results_and_advance(state);
                None
            }
            UiEvent::Text('c' | 'C') => {
                self.controller.ui_state_mut().toggle_compact();
                None
            }
            UiEvent::Text('s' | 'S') => {
                self.controller.ui_state_mut().toggle_stats();
                None
            }
            UiEvent::Text('k' | 'K') => {
                let ko = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| {
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
                    .unwrap_or(false);
                if ko {
                    let round1 = state
                        .active_competition
                        .as_ref()
                        .and_then(|active| {
                            active
                                .individual()
                                .map(|c| c.phase() == CompetitionPhase::Round1Results)
                        })
                        .unwrap_or(false);
                    self.controller.ui_state_mut().toggle_ko_pairs(round1);
                }
                None
            }
            UiEvent::KeyDown(Key::Left) => {
                self.controller.ui_state_mut().prev_page();
                None
            }
            UiEvent::KeyDown(Key::Enter) => {
                let is_season_complete = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| {
                        active
                            .individual()
                            .map(|c| c.phase() == CompetitionPhase::SeasonComplete)
                    })
                    .unwrap_or(false);
                if is_season_complete {
                    self.save_competition_results(state);
                    return Some(RouteTarget::Back);
                }
                self.dismiss_results_and_advance(state);
                None
            }
            _ => None,
        }
    }
}

fn phase_label(resources: &ResourcesRef, phase: CompetitionPhase) -> String {
    let lang = &resources.langbase;
    match phase {
        CompetitionPhase::Training(n) => format!("{} {}", lang.tr(52), n),
        CompetitionPhase::Qualification => lang.tr(53).to_string(),
        CompetitionPhase::Round1 => lang.tr(54).to_string(),
        CompetitionPhase::Round2 => lang.tr(55).to_string(),
        _ => lang.tr(51).to_string(),
    }
}

fn format_wc_best_result(points: i32, rank: usize) -> String {
    format!("{points} ({rank}.)")
}

fn format_four_hills_best_result(points: f64, rank: usize) -> String {
    format!("{} ({}.)", format_decimal(points), rank)
}
