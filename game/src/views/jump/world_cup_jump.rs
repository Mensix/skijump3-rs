use crate::competition::types::{CompetitionPhase, CupStyle, DID_NOT_START_SCORE};
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
use crate::gfx::palette::{apply_menu_tint, FONT_GOLD};
use crate::gfx::sprites;
use crate::jump::types::{FallType, JumpPhase};
use crate::jump::JumpParticipant;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Jump,
    Results,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultScreen {
    List,
    KoPairs(bool),
    Stats,
}

/// Default key names matching our input bindings (K[1..5]).
const KEY_NAMES: [&str; 5] = ["UP", "RIGHT", "LEFT", "T", "R"];

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    last_event: Cell<usize>,
    result_acknowledged: Cell<bool>,
    display_page: Cell<usize>,
    render_mode: Cell<RenderMode>,
    result_screen: Cell<ResultScreen>,
    compact_list: Cell<bool>,
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
            resources,
            store,
            scene,
            last_event: Cell::new(0),
            result_acknowledged: Cell::new(false),
            display_page: Cell::new(0),
            render_mode: Cell::new(RenderMode::Jump),
            result_screen: Cell::new(ResultScreen::List),
            compact_list: Cell::new(false),
        }
    }

    fn record_finished_human_jump(&self) {
        let outcome = self.scene.outcome();
        if outcome.is_none() || !self.result_acknowledged.get() {
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
        self.result_acknowledged.set(false);
    }

    fn handle_human_jump(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = self.scene.participant_id() != participant.id
            || self.scene.hill_idx() != hill_idx
            || (self.scene.outcome().is_some() && self.result_acknowledged.get());
        if needs_rebuild {
            self.result_acknowledged.set(false);
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
                match self.result_screen.get() {
                    ResultScreen::KoPairs(show_results) => {
                        return competition_results::render_ko_pairs(
                            c,
                            &self.resources,
                            show_results,
                        );
                    }
                    ResultScreen::Stats => {
                        return competition_results::render_stats_page(
                            c,
                            &self.resources,
                            self.display_page.get(),
                        );
                    }
                    ResultScreen::List => {}
                }
                let page_data = if self.compact_list.get() {
                    competition_results::build_compact_results_page(c)
                } else {
                    competition_results::build_results_page(c, self.display_page.get())
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
            self.render_mode.set(RenderMode::Done);
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
                self.render_mode.set(RenderMode::Jump);
                self.result_screen.set(ResultScreen::List);
            }
            WorldCupCommand::ShowResults => {
                self.select_default_result_screen();
                self.render_mode.set(RenderMode::Results);
            }
            WorldCupCommand::Done => {
                self.render_mode.set(RenderMode::Done);
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

    /// Pascal info cycle: l = frame_counter % 438.
    /// Ranges (SJ3.PAS:965-996):
    ///   0-130:   hill record (WC cycle) or top-5 event (general)
    ///   146-276: WC standings or hill record
    ///   292-422: WC standings if available
    ///   437:     reset l
    fn info_cycle_elements(&self, els: &mut Vec<Element>) {
        let fc = self.scene.frame_counter();
        let l = fc % 438;
        let (has_wc, top5_wc, _top5_event, gap_label, gap_pts) = self
            .store
            .competition
            .try_with(|c| {
                let has_wc = c
                    .overall_standings()
                    .first()
                    .is_some_and(|p| p.wc_points > 0);
                let top5_wc: Vec<_> = c
                    .overall_standings()
                    .iter()
                    .filter(|p| p.wc_points > 0)
                    .take(5)
                    .map(|p| (p.display_name().to_string(), p.wc_points))
                    .collect();
                let top5_event: Vec<_> = c
                    .event_standings()
                    .iter()
                    .filter(|p| p.points != DID_NOT_START_SCORE)
                    .take(5)
                    .map(|p| (p.display_name().to_string(), p.points))
                    .collect();
                let (gap_label, gap_pts) = {
                    let standings = c.event_standings();
                    let leader_pts = standings.first().map(|p| p.points).unwrap_or(0);
                    let is_round2 = c.phase() == CompetitionPhase::Round2;
                    let label_idx = if is_round2 { 63 } else { 62 };
                    let label = self.resources.langbase.lstr(label_idx).to_string();
                    (label, leader_pts)
                };
                (has_wc, top5_wc, top5_event, gap_label, gap_pts)
            })
            .unwrap_or((false, vec![], vec![], String::new(), 0));

        let panel_x = 227;

        // Panel always visible. Hill record by default;
        // WC top-5 during designated cycle segments when available.
        if has_wc && !top5_wc.is_empty() && ((146..=276).contains(&l) || (292..=422).contains(&l)) {
            self.info_panel_with_top5(els, panel_x, &top5_wc, true);
        } else {
            self.info_panel_with_hill_record(els, panel_x);
        }

        // Round 2 gap/position overlay at (308, 62)
        if !gap_label.is_empty() && gap_pts > 0 {
            let current_id = self.scene.participant_id();
            let current_pts = self
                .store
                .competition
                .try_with(|c| {
                    c.event_standings()
                        .iter()
                        .find(|p| p.id == current_id)
                        .map(|p| p.points)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            if current_pts > 0 && gap_pts > 0 {
                let diff = gap_pts - current_pts;
                if diff > 0 {
                    els.push(Element::text(
                        format!("{}: {}", gap_label, diff),
                        308,
                        62,
                        FONT_GOLD,
                        false,
                    ));
                }
            }
        }
    }

    /// Pascal drawkeymap: key binding hints shown when jumper is on bar.
    fn drawkeymap_elements(&self, els: &mut Vec<Element>) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        els.push(Element::text(
            self.resources.langbase.lstr(330),
            308,
            9,
            FONT_GOLD,
            true,
        ));
        for i in 1..=5 {
            els.push(Element::text(
                format!(
                    "{}: {}",
                    self.resources.langbase.lstr(330 + i),
                    KEY_NAMES[i - 1]
                ),
                308,
                i as i32 * 10 + 9,
                FONT_GOLD,
                true,
            ));
        }
    }

    fn info_panel_with_hill_record(&self, els: &mut Vec<Element>, panel_x: i32) {
        els.push(Element::sprite(
            sprites::Sprite::InfoPanel as u16,
            panel_x,
            2,
        ));
        let hill_idx = self.scene.hill_idx();
        if let Some(hill) = self.resources.hills.hill(hill_idx) {
            els.push(Element::text(
                format!("{} K{}", hill.name, hill.kr),
                308,
                9,
                FONT_GOLD,
                true,
            ));
        }
        if let Some(record) = self.store.records.borrow().hill_record(hill_idx) {
            if record.len > 0 {
                els.push(Element::text(&record.name, 308, 19, FONT_GOLD, true));
                els.push(Element::text(
                    format!("{:.1}m", record.len as f64 / 10.0),
                    308,
                    29,
                    FONT_GOLD,
                    true,
                ));
            }
        }
    }

    fn info_panel_with_top5(
        &self,
        els: &mut Vec<Element>,
        panel_x: i32,
        top5: &[(String, i32)],
        is_wc: bool,
    ) {
        els.push(Element::sprite(
            sprites::Sprite::InfoPanel as u16,
            panel_x,
            2,
        ));
        if is_wc {
            els.push(Element::text(
                self.resources.langbase.lstr(70),
                308,
                9,
                FONT_GOLD,
                true,
            ));
        } else {
            let hill_idx = self.scene.hill_idx();
            if let Some(hill) = self.resources.hills.hill(hill_idx) {
                els.push(Element::text(
                    format!("{} K{}", hill.name, hill.kr),
                    308,
                    9,
                    FONT_GOLD,
                    true,
                ));
            }
        }
        let leader_pts = top5.first().map(|(_, pts)| *pts).unwrap_or(0);
        for (i, (name, pts)) in top5.iter().enumerate() {
            let val = if is_wc && i > 0 && leader_pts > 0 {
                format!("{}", pts - leader_pts)
            } else {
                pts.to_string()
            };
            els.push(Element::text(
                format!("{name}${val}"),
                308,
                13 + i as i32 * 7,
                FONT_GOLD,
                true,
            ));
        }
    }
}

impl WorldCupJumpView {
    fn select_default_result_screen(&self) {
        self.result_screen.set(
            self.store
                .competition
                .try_with(|c| {
                    if c.style() == CupStyle::FourHills {
                        match c.phase() {
                            CompetitionPhase::QualificationResults => ResultScreen::KoPairs(false),
                            CompetitionPhase::Round1Results => ResultScreen::KoPairs(true),
                            _ => ResultScreen::List,
                        }
                    } else {
                        ResultScreen::List
                    }
                })
                .unwrap_or(ResultScreen::List),
        );
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        self.record_finished_human_jump();
        self.drive_competition();
        if self.render_mode.get() == RenderMode::Jump {
            self.scene.update();
        }
    }

    fn elements(&self) -> Vec<Element> {
        match self.render_mode.get() {
            RenderMode::Jump => {
                let mut els = self.scene.elements();
                match self.scene.phase() {
                    // Key bindings during OnBar (jumper sitting at gate)
                    Some(JumpPhase::OnBar) => self.drawkeymap_elements(&mut els),
                    // Info cycle during flight
                    Some(JumpPhase::Inrun | JumpPhase::Flight) => {
                        self.info_cycle_elements(&mut els)
                    }
                    _ => {}
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
        if self.scene.outcome().is_some() && !self.result_acknowledged.get() {
            if matches!(event, Event::Keyboard(Key::Enter | Key::Escape)) {
                self.result_acknowledged.set(true);
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
            let tint = self
                .store
                .competition
                .try_with(|c| match c.phase() {
                    CompetitionPhase::WorldCupStandings | CompetitionPhase::SeasonComplete => 5,
                    _ => 0,
                })
                .unwrap_or(0);
            apply_menu_tint(palette, 3, tint);
            return;
        }
        self.scene.apply_palette(palette);
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.display_page.get() > 0 {
            return true;
        }
        self.store
            .competition
            .try_with(|c| {
                matches!(
                    c.phase(),
                    CompetitionPhase::QualificationResults
                        | CompetitionPhase::Round1Results
                        | CompetitionPhase::Round2Results
                        | CompetitionPhase::WorldCupStandings
                        | CompetitionPhase::SeasonComplete
                ) || matches!(
                    c.phase(),
                    CompetitionPhase::Qualification
                        | CompetitionPhase::Round1
                        | CompetitionPhase::Round2
                ) && c.current_jumper().is_none()
            })
            .unwrap_or(false)
    }

    fn handle_result_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                let page = self.display_page.get();
                let total = match self.result_screen.get() {
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
                    ResultScreen::List if self.compact_list.get() => 1,
                    ResultScreen::List => self
                        .store
                        .competition
                        .try_with(competition_results::total_pages)
                        .unwrap_or(0),
                };
                if page + 1 < total {
                    self.display_page.set(page + 1);
                }
                None
            }
            Event::Keyboard(Key::Char('c') | Key::Char('C')) => {
                self.compact_list.set(!self.compact_list.get());
                self.result_screen.set(ResultScreen::List);
                self.display_page.set(0);
                None
            }
            Event::Keyboard(Key::Char('s') | Key::Char('S')) => {
                self.result_screen.set(match self.result_screen.get() {
                    ResultScreen::Stats => ResultScreen::List,
                    _ => ResultScreen::Stats,
                });
                self.display_page.set(0);
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
                    self.result_screen.set(match self.result_screen.get() {
                        ResultScreen::KoPairs(_) => ResultScreen::List,
                        _ => ResultScreen::KoPairs(
                            self.store
                                .competition
                                .try_with(|c| c.phase() == CompetitionPhase::Round1Results)
                                .unwrap_or(false),
                        ),
                    });
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                let page = self.display_page.get();
                if page > 0 {
                    self.display_page.set(page - 1);
                }
                None
            }
            Event::Keyboard(Key::Escape | Key::Enter) => {
                if self
                    .store
                    .competition
                    .try_with(|c| c.phase() == CompetitionPhase::SeasonComplete)
                    .unwrap_or(false)
                {
                    return Some(RouteTarget::Back);
                }
                self.display_page.set(0);
                self.result_screen.set(ResultScreen::List);
                self.store.competition.try_with_mut(|c| c.advance());
                None
            }
            _ => None,
        }
    }
}
