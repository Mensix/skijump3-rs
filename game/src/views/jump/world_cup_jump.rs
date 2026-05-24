use crate::competition::machine::Competition;
use crate::competition::scoring::wc_points_for_rank;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::controllers::competition_ui::{CompetitionUiState, RenderMode, ResultScreen};
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
use crate::gfx::palette::{apply_menu_tint, FONT_GREET};
use crate::jump::types::{FallType, JumpPhase};
use crate::jump::JumpParticipant;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_tenths;
use crate::views::jump::competition_overlay::CompetitionOverlay;
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Blinker, Element, Event, Key, View};
use std::cell::Cell;

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    last_event: Cell<usize>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    blinker: Blinker,
    profiles_saved: Cell<bool>,
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
            crate::jump::JumpPolicy::competition(),
        );
        Self {
            resources: ResourcesRef::clone(&resources),
            store: StoreRef::clone(&store),
            scene,
            last_event: Cell::new(0),
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(
                ResourcesRef::clone(&resources),
                StoreRef::clone(&store),
            ),
            blinker: Blinker::new(),
            profiles_saved: Cell::new(false),
        }
    }

    fn record_finished_human_jump(&self) {
        let outcome = self.scene.outcome();
        if outcome.is_none()
            || !self.ui_state.is_result_acknowledged()
            || self.ui_state.is_outcome_recorded()
        {
            return;
        }
        let outcome = outcome.unwrap();
        if !self
            .store
            .competition
            .try_with(|c| c.is_human_current())
            .unwrap_or(false)
        {
            return;
        }
        self.store.competition.try_with_mut(|c| {
            if outcome.fall_type == FallType::Crash {
                c.injure_current(3);
            }
            c.record_jump(outcome.score, outcome.distance);
        });
        self.ui_state.mark_outcome_recorded();
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

    fn results_page(&self) -> Vec<Element> {
        self.store
            .competition
            .try_with(|c| {
                match self.ui_state.current_screen() {
                    ResultScreen::KoPairs(show_results) => {
                        let show_cursor = self.blinker.visible(10, 10);
                        return competition_results::render_ko_pairs(
                            c,
                            &self.resources,
                            show_results,
                            show_cursor,
                        );
                    }
                    ResultScreen::Stats => {
                        return competition_results::render_stats_page(
                            c,
                            &self.resources,
                            self.ui_state.current_page(),
                        );
                    }
                    ResultScreen::List => {}
                }
                let page_data = if self.ui_state.is_compact() {
                    competition_results::build_compact_results_page(c)
                } else {
                    competition_results::build_results_page(c, self.ui_state.current_page())
                };
                let mut els = competition_results::render_results_page(&page_data, &self.resources);
                els.extend(competition_results::render_header(c, &self.resources));
                els
            })
            .unwrap_or_else(|| vec![Element::fillbox(0, 0, 320, 200, 0)])
    }

    fn drive_competition(&self) {
        let command = self.store.competition.try_with_mut(|c| {
            let mut simulate_computer = |participant: JumpParticipant,
                                         hill_idx: usize|
             -> crate::jump::types::JumpOutcome {
                self.scene.simulate_hidden(participant, hill_idx)
            };
            world_cup_flow::drive(c, &self.last_event, &mut simulate_computer)
        });

        let Some(command) = command else {
            self.ui_state.enter_done();
            return;
        };

        match command {
            WorldCupCommand::HumanJump {
                participant,
                hill_idx,
                phase,
                is_new_event,
            } => {
                if is_new_event {
                    JumpScene::setup_event(&self.store);
                }
                let phase_label = Self::phase_label(&self.resources, phase);
                self.handle_human_jump(participant, hill_idx, phase_label);
                self.ui_state.enter_jump();
            }
            WorldCupCommand::ShowResults => {
                // Only initialize result UI on first entry — not every
                // frame, otherwise paging/toggles are instantly reset.
                if self.ui_state.render_mode() != RenderMode::Results {
                    self.select_default_result_screen();
                    self.ui_state.enter_results();
                }
            }
            WorldCupCommand::Done => {
                self.save_competition_results();
                self.ui_state.enter_done();
            }
        }
    }

    fn save_competition_results(&self) {
        if self.profiles_saved.replace(true) {
            return;
        }
        self.store.competition.try_with(|c| {
            let style = c.style();
            let overall = c.overall_standings();
            let mut profiles = self.store.profiles.borrow_mut();

            for p in &overall {
                let Some(pidx) = p.profile_idx else {
                    continue;
                };
                let Some(profile) = profiles.profiles.get_mut(pidx) else {
                    continue;
                };

                match style {
                    CupStyle::WorldCup => {
                        profile.world_cups += 1;
                        let my_points = p.points.unwrap_or(0);
                        let event_rank = event_rank_by_points(c, my_points);
                        let pts = wc_points_for_rank(event_rank);
                        if pts >= profile.bestpoints as i32 {
                            profile.bestpoints = pts as usize;
                            profile.best_result = format_wc_best_result(pts, event_rank);
                        }
                    }
                    CupStyle::FourHills => {
                        if p.four_hills_points >= profile.best4points as i32 {
                            profile.best4points = p.four_hills_points as usize;
                            profile.best_4h_result =
                                format_four_hills_best_result(p.four_hills_points, p.rank);
                        }
                    }
                    CupStyle::CustomCup | CupStyle::TeamCup => {}
                }
            }
        });
        self.resources
            .save_manager
            .save_players(&self.store.profiles.borrow());
        if let Ok(records) = self.store.records.try_borrow() {
            self.resources.save_manager.save_records(&records);
        }
    }

    /// Pascal: rank calculation — counts participants with points <= jumper's total.
    /// Shows `($X.)` at (255,45), left of the score at (308,45).
    fn rank_element(&self) -> Option<Element> {
        let outcome = self.scene.outcome()?;
        if self.scene.phase() != Some(JumpPhase::Result) {
            return None;
        }
        let own_id = self.scene.participant_id();
        self.store.competition.try_with(|c| {
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
            Element::right_text(format!("(${}.)", rank), 255, 45, FONT_GREET)
        })
    }

    fn select_default_result_screen(&self) {
        if let Some((style, phase)) = self.store.competition.try_with(|c| (c.style(), c.phase())) {
            self.ui_state
                .select_default_screen(style == CupStyle::FourHills, phase);
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
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        self.record_finished_human_jump();
        self.drive_competition();
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
                    .competition
                    .try_with(|c| {
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
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
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

    fn render_snow(&self, framebuffer: &mut [u8]) {
        self.scene.render_snow(framebuffer);
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if self.is_result_display_state() {
            // Pascal: for style 1 screens, always MuutaMenu(3, 0) (gray base)
            apply_menu_tint(palette, 3, 0);
            // Pascal: color-specific tint on index 1 — col=5 (red) for WC standings
            let tint = self
                .store
                .competition
                .try_with(|c| {
                    if c.phase() == CompetitionPhase::WorldCupStandings
                        || c.phase() == CompetitionPhase::SeasonComplete
                    {
                        if c.style() == CupStyle::FourHills {
                            1
                        } else {
                            5
                        }
                    } else if c.phase() == CompetitionPhase::FourHillsStandings {
                        1
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            if tint > 0 {
                apply_menu_tint(palette, 1, tint);
            }
            return;
        }
        self.scene.apply_palette(palette);
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.ui_state.has_page() {
            return true;
        }
        self.store
            .competition
            .try_with(|c| {
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
                        .competition
                        .try_with(|c| {
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
                        .competition
                        .try_with(competition_results::total_pages)
                        .unwrap_or(0),
                };
                if self.ui_state.next_page(total) {
                    return None;
                }
                // Pascal WaitForKey(0): any key on the last entry exits the list
                self.blinker.reset();
                self.ui_state.dismiss_results();
                self.store.competition.try_with_mut(|c| c.advance());
                self.drive_competition();
                None
            }
            Event::Keyboard(Key::Char('c') | Key::Char('C')) => {
                self.ui_state.toggle_compact();
                None
            }
            Event::Keyboard(Key::Char('s') | Key::Char('S')) => {
                self.ui_state.toggle_stats();
                None
            }
            Event::Keyboard(Key::Char('k') | Key::Char('K')) => {
                let ko = self
                    .store
                    .competition
                    .try_with(|c| {
                        c.style() == CupStyle::FourHills
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
                        .competition
                        .try_with(|c| c.phase() == CompetitionPhase::Round1Results)
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
                    .competition
                    .try_with(|c| c.phase() == CompetitionPhase::SeasonComplete)
                    .unwrap_or(false);
                if is_season_complete {
                    self.save_competition_results();
                    return Some(RouteTarget::Back);
                }
                self.blinker.reset();
                self.ui_state.dismiss_results();
                self.store.competition.try_with_mut(|c| c.advance());
                self.drive_competition();
                None
            }
            _ => None,
        }
    }
}

/// Pascal: `txt(mcpisteet[who])+' ('+str1+')'` where str1 is `sija[who]+'.'`
/// for the final event. Same applies to best4 result with `txtp` for tenths.
fn format_wc_best_result(points: i32, rank: usize) -> String {
    format!("{} ({}.)", points, rank)
}

fn format_four_hills_best_result(points_tenths: i32, rank: usize) -> String {
    format!("{} ({}.)", format_tenths(points_tenths), rank)
}

/// Tie-aware event rank: 1 + count of participants with strictly higher points.
fn event_rank_by_points(competition: &Competition, points: i32) -> usize {
    1 + competition
        .event_standings()
        .iter()
        .filter(|p| p.points.unwrap_or(i32::MIN) > points)
        .count()
}
