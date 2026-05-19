use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
use crate::gfx::palette::{apply_menu_tint, FONT_GOLD, FONT_GREET};
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
const KEY_NAMES: [&str; 5] = ["ARROW UP", "ARROW RIGHT", "ARROW LEFT", "T", "R"];

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
}

impl WorldCupJumpView {
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
            Element::text(format!("(${}.)", rank), 255, 45, FONT_GREET, true)
        })
    }

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
                // Key bindings during OnBar (jumper sitting at gate)
                // Also shown during Disqualified (matches Pascal drawscreen
                // capture of the last OnBar frame before the DQ overlay).
                // Nothing extra during Inrun/Flight — only wind gauge from scene.elements()
                if let Some(JumpPhase::OnBar | JumpPhase::Disqualified) = self.scene.phase() {
                    self.drawkeymap_elements(&mut els);
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
        if self.scene.outcome().is_some() && !self.result_acknowledged.get() {
            let is_dq = self.scene.phase() == Some(JumpPhase::Disqualified);
            let accepted = if is_dq {
                // Pascal waitforkey: ANY key dismisses the DQ screen
                matches!(event, Event::Keyboard(_))
            } else {
                matches!(event, Event::Keyboard(Key::Enter | Key::Escape))
            };
            if accepted {
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
