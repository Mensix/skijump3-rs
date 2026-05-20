use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
use crate::gfx::palette::{apply_menu_tint, FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::gfx::sprites;
use crate::jump::types::{FallType, JumpPhase};
use crate::jump::JumpParticipant;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::Cell;

/// What overlay to draw on top of the jump scene during competition phases.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OverlayKind {
    None,
    Keymap,
    CyclingInfo,
    Round2CyclingWithInfoBox,
}

fn fmt_tenths(val: i32) -> String {
    let sign = if val < 0 { "-" } else { "" };
    let abs = val.abs();
    format!("{}{}.{}", sign, abs / 10, abs % 10)
}

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
    outcome_recorded: Cell<bool>,
    first_human_onbar: Cell<bool>,
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
            outcome_recorded: Cell::new(false),
            first_human_onbar: Cell::new(true),
            display_page: Cell::new(0),
            render_mode: Cell::new(RenderMode::Jump),
            result_screen: Cell::new(ResultScreen::List),
            compact_list: Cell::new(false),
        }
    }

    fn record_finished_human_jump(&self) {
        let outcome = self.scene.outcome();
        if outcome.is_none() || !self.result_acknowledged.get() || self.outcome_recorded.get() {
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
        self.outcome_recorded.set(true);
        self.first_human_onbar.set(false);
    }

    fn handle_human_jump(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = self.scene.participant_id() != participant.id
            || self.scene.hill_idx() != hill_idx
            || self.outcome_recorded.get()
            || (self.scene.outcome().is_some() && self.result_acknowledged.get());
        if needs_rebuild {
            self.result_acknowledged.set(false);
            self.outcome_recorded.set(false);
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

impl WorldCupJumpView {
    /// Pure decision: what overlay kind to draw for the current state.
    fn overlay_kind(&self) -> Option<OverlayKind> {
        let scene_phase = self.scene.phase()?;
        if scene_phase == JumpPhase::Disqualified {
            return Some(OverlayKind::None);
        }

        let (phase, participant, style) = self.store.competition.try_with(|c| {
            let idx = c.current_jumper().unwrap_or(0);
            let p = c.participant(idx);
            (c.phase(), p.clone(), c.style())
        })?;

        let first_event = self.store.competition.try_with(|c| c.current_event == 0).unwrap_or(false);
        let show_keymap = self.first_human_onbar.get() && first_event && !participant.is_computer;

        match (phase, scene_phase) {
            (CompetitionPhase::Round2, JumpPhase::Info) if !matches!(style, CupStyle::CustomCup) => {
                Some(OverlayKind::Round2CyclingWithInfoBox)
            }
            (CompetitionPhase::Qualification, JumpPhase::Info) => {
                Some(if show_keymap { OverlayKind::Keymap } else { OverlayKind::CyclingInfo })
            }
            (CompetitionPhase::Qualification, JumpPhase::OnBar) if show_keymap => {
                Some(OverlayKind::Keymap)
            }
            _ => Some(OverlayKind::None),
        }
    }

    /// Pascal OnBar draw sequence:
    ///   Info phase (first loop) — InfoPanel with cycling info or keymap
    ///   OnBar (second loop, sitting on bar) — only keymap for first event's first human
    ///   Disqualified — only DQ info bar (provided by presentation::dq_elements)
    fn onbar_overlay(&self, els: &mut Vec<Element>) {
        let kind = match self.overlay_kind() {
            Some(k) => k,
            None => return,
        };
        let frame_counter = self.scene.frame_counter();
        let Some((_, _, hill_idx, _)) = self.store.competition.try_with(|c| {
            let idx = c.current_jumper().unwrap_or(0);
            let p = c.participant(idx);
            Some((c.phase(), p.clone(), c.current_hill(), c.style()))
        }).flatten() else { return };

        let participant = self.store.competition.try_with(|c| {
            let idx = c.current_jumper().unwrap_or(0);
            Some(c.participant(idx).clone())
        }).flatten();

        match kind {
            OverlayKind::None => {}
            OverlayKind::Keymap => self.drawkeymap_elements(els),
            OverlayKind::CyclingInfo => self.cycling_info_elements(els, frame_counter, hill_idx),
            OverlayKind::Round2CyclingWithInfoBox => {
                self.cycling_info_elements(els, frame_counter, hill_idx);
                if let Some(ref p) = participant {
                    self.round2_jumper_info_box(els, p);
                }
            }
        }
    }

    /// Pascal: JumperInfoBox at (3,150) during Round 2 OnBar showing R1 total + length.
    fn round2_jumper_info_box(&self, els: &mut Vec<Element>, participant: &crate::competition::types::Participant) {
        els.push(Element::sprite(sprites::Sprite::JumperInfoBox as u16, 3, 150));

        let phase_label = Self::phase_label(&self.resources, CompetitionPhase::Round2);
        let label56 = self.resources.langbase.lstr(56);
        let label56_w = self.resources.font.string_width(label56) as i32;

        els.push(Element::text(phase_label, 12, 160, FONT_GREET, false));
        els.push(Element::text(label56, 12, 172, FONT_GREET, false));
        let rank = participant.round1_rank;
        let name = if rank > 0 {
            format!("{} ({}.)", participant.display_name(), rank)
        } else {
            participant.display_name().to_string()
        };
        els.push(Element::text(name, 12 + label56_w, 172, FONT_DEFAULT, false));

        let r1text = format!(
            "{} ({}µ)",
            fmt_tenths(participant.round1_score),
            fmt_tenths(participant.round1_len)
        );
        els.push(Element::text(r1text, 14 + label56_w, 179, FONT_HELP, false));
        els.push(Element::text(
            self.resources.langbase.lstr(59),
            12,
            191,
            FONT_HELP,
            false,
        ));
    }

    /// Pascal drawinfo cycling: event top5 (0..130) / hill record (146..276) / WC standings (292..)
    /// Pascal drawinfo: cycling info with 131-frame content windows separated
    /// by 15-frame blank gaps. Each content block pushes the InfoPanel sprite;
    /// gap frames push only the sprite to mask the static text from
    /// presentation::info_elements.
    /// WC standings (292..422) are only shown when the WC leader has points
    /// (Pascal `if (mcpisteet[mcluett[1]]>0)`); otherwise Pascal resets the
    /// counter (`l:=0`) so the cycle is top5(130) → gap(15) → hr(130) → gap(15)
    /// = 292 frames. We use modulo 292 in that case.
    fn cycling_info_elements(&self, els: &mut Vec<Element>, frame_counter: i32, hill_idx: usize) {
        let has_wc_leader = self.store.competition.try_with(|c| {
            c.overall_standings()
                .first()
                .map(|p| p.wc_points > 0)
                .unwrap_or(false)
        }).unwrap_or(false);

        let cycle = if has_wc_leader { 438 } else { 292 };
        let phase = (frame_counter as usize) % cycle;

        if phase <= 130 {
            self.top5_event_elements(els);
        } else if (146..=276).contains(&phase) {
            self.hill_info_elements(els, hill_idx);
        } else if has_wc_leader && (292..=422).contains(&phase) {
            self.wc_standings_elements(els);
        } else {
            els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        }
    }

    /// Pascal drawtop5info: hill name + top 5 event points with gap behind leader
    fn top5_event_elements(&self, els: &mut Vec<Element>) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        let hill_name_k = self
            .store
            .competition
            .try_with(|c| {
                self.resources
                    .hills
                    .hill(c.current_hill())
                    .map(|h| format!("{} K{}", h.name, h.kr))
            })
            .flatten()
            .unwrap_or_default();
        els.push(Element::text(hill_name_k, 308, 9, FONT_GOLD, true));

        self.store.competition.try_with(|c| {
            let standings = c.event_standings();
            for (i, p) in standings.iter().enumerate().take(5) {
                if let Some(pts) = p.points {
                    if pts > 0 {
                        els.push(Element::text(
                            format!("{}  {}", p.name, fmt_tenths(pts)),
                            308,
                            20 + i as i32 * 7,
                            FONT_GOLD,
                            true,
                        ));
                    }
                }
            }

            // Pascal: gap-to-leader line at y=62 (behind/lead label)
            if let Some(current) = c.current_jumper() {
                let pel = c.participant(current);
                let leader_pts = standings.first().and_then(|p| p.points).unwrap_or(0);
                let current_pts = pel.points.unwrap_or(0);
                let temp = leader_pts - current_pts;
                if temp > 0 {
                    let label = self.resources.langbase.lstr(62);
                    els.push(Element::text(
                        format!("{}: {}", label, fmt_tenths(temp + 1)),
                        308,
                        62,
                        FONT_GOLD,
                        true,
                    ));
                }
            }
        });
    }

    /// Pascal drawhrinfo: hill record name + distance
    fn hill_info_elements(&self, els: &mut Vec<Element>, hill_idx: usize) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        let hill_name_k = self
            .resources
            .hills
            .hill(hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        els.push(Element::text(
            hill_name_k,
            308,
            9,
            FONT_GOLD,
            true,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(65).to_string(),
            308,
            19,
            FONT_GOLD,
            true,
        ));
        let records = self.store.records.borrow();
        let record = records.hill_record(hill_idx);
        if let Some(r) = record {
            if r.len > 0 {
                els.push(Element::text(r.name.clone(), 308, 29, FONT_GOLD, true));
                els.push(Element::text(
                    format!("{:.1}m", r.len as f64 / 10.0),
                    308,
                    39,
                    FONT_GOLD,
                    true,
                ));
            }
        }
    }

    /// Pascal drawwcinfo: top 5 World Cup / season standings.
    /// Pascal renders name + '$' (space) + points as one right-aligned string.
    /// diffwc defaults to false — full points shown, not gaps.
    fn wc_standings_elements(&self, els: &mut Vec<Element>) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        els.push(Element::text(
            self.resources.langbase.lstr(70).to_string(),
            308,
            9,
            FONT_GOLD,
            true,
        ));

        self.store.competition.try_with(|c| {
            let standings = c.overall_standings();
            for (i, p) in standings.iter().enumerate().take(5) {
                if p.wc_points > 0 {
                    // Pascal: nimet[who] + '$' + txt(mcpisteet[who]) — raw points
                    let s = format!("{}  {}", p.name, p.wc_points);
                    els.push(Element::text(s, 308, 20 + i as i32 * 7, FONT_GOLD, true));
                }
            }
        });
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
                // Suppress static InfoPanel text when the overlay provides cycling info.
                let hide = self.store.competition.try_with(|c| {
                    matches!(c.phase(), CompetitionPhase::Round2)
                        && !matches!(c.style(), CupStyle::CustomCup)
                }).unwrap_or(false);
                self.scene.set_hide_info_panel_text(hide);
                let mut els = self.scene.elements();
                self.onbar_overlay(&mut els);
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
            // Pascal: for style 1 screens, always MuutaMenu(3, 0) (gray base)
            apply_menu_tint(palette, 3, 0);
            // Pascal: color-specific tint on index 1 — col=5 (red) for WC standings
            let tint = self
                .store
                .competition
                .try_with(|c| match c.phase() {
                    CompetitionPhase::WorldCupStandings | CompetitionPhase::SeasonComplete => 5,
                    _ => 0,
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
                    return None;
                }
                // Pascal WaitForKey(0): any key on the last entry exits the list
                self.display_page.set(0);
                self.result_screen.set(ResultScreen::List);
                self.store.competition.try_with_mut(|c| c.advance());
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
