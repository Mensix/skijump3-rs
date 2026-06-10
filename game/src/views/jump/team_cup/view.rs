use super::session::{TeamCupSessionController, TeamCupUiCommand};
use crate::competition::runtime::CompetitionRuntime;
use crate::competition::team_cup::types::{TeamCupResultsKind, TeamCupStandingsKind};
use crate::components::screen::{self, new_screen_with_bg};
use crate::gfx::palette::{BG_TEAMCUP, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use crate::jump::hud;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::layout::shorten_name;
use crate::views::jump::competition::flow::handle_human_jump;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::input::{JumpInputAction, JumpInputController};
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
    phase: ViewPhase,
    blinker: Blinker,
    controller: TeamCupSessionController,
    team_names: Vec<String>,
    name_buffer: String,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let names: Vec<String> = store
            .try_with_team_cup(|tc| {
                tc.teams
                    .iter()
                    .filter(|t| t.is_human_team)
                    .map(|t| t.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        let name_buffer = names.first().cloned().unwrap_or_default();
        Self {
            resources: resources.clone(),
            store: store.clone(),
            scene: None,
            ui_state: CompetitionUiState::new(),
            phase: ViewPhase::NamingTeam(0),
            blinker: Blinker::new(),
            controller: TeamCupSessionController::new(resources, store),
            team_names: names,
            name_buffer,
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn get_team_x(&self, team_idx: usize) -> i32 {
        if team_idx == 0 {
            30
        } else {
            160
        }
    }

    fn naming_elements(&self) -> Vec<Element> {
        let mut els = new_screen_with_bg(1, BG_TEAMCUP);

        drop_team_cup_header(&mut els, &self.resources, &self.store);

        for n in 0..self.team_names.len() {
            let xx = self.get_team_x(n);
            let is_current = matches!(self.phase, ViewPhase::NamingTeam(t) if t == n);

            jumper_names_els(&mut els, &self.store, n, xx);

            if is_current {
                // "Please Name Team N:" (white, color 240)
                els.push(Element::text(
                    format!("{} {}:", self.resources.langbase.lstr(113), n + 1),
                    xx,
                    30,
                    FONT_DEFAULT,
                    false,
                ));

                // Input field: fillbox(xx-2,40,xx+maxlength+2,49,242)
                els.push(Element::fillbox(xx - 2, 40, 125, 10, BLACK));
                els.push(Element::text(
                    self.name_buffer.clone(),
                    xx,
                    42,
                    FONT_DEFAULT,
                    false,
                ));
                if self.cursor_visible {
                    let cw = self.resources.font.string_width(&self.name_buffer) as i32;
                    // givech underscore cursor at (xx+cx, yy+6), 5×1
                    els.push(Element::fillbox(xx + cw, 48, 5, 1, FONT_DEFAULT));
                }
            } else {
                // Already named: clear area (Pascal FillBox(xx-10,30,xx+124,54,243))
                els.push(Element::fillbox(xx - 10, 30, 135, 25, BG_TEAMCUP));
                els.push(Element::fill_area(63));

                // "Team N:" (gray, color 241) + name (white)
                els.push(Element::text(
                    format!("{} {}:", self.resources.langbase.lstr(114), n + 1),
                    xx,
                    30,
                    FONT_HELP,
                    false,
                ));
                els.push(Element::text(
                    self.team_names[n].clone(),
                    xx,
                    42,
                    FONT_DEFAULT,
                    false,
                ));
            }
        }

        els
    }

    fn ready_elements(&self) -> Vec<Element> {
        let mut els = new_screen_with_bg(1, BG_TEAMCUP);

        drop_team_cup_header(&mut els, &self.resources, &self.store);

        for n in 0..self.team_names.len() {
            let xx = self.get_team_x(n);

            jumper_names_els(&mut els, &self.store, n, xx);

            // Clear area (Pascal FillBox(xx-10,30,xx+124,54,243))
            els.push(Element::fillbox(xx - 10, 30, 135, 25, BG_TEAMCUP));
            els.push(Element::fill_area(63));

            // "Team N:" (gray) + name (white)
            els.push(Element::text(
                format!("{} {}:", self.resources.langbase.lstr(114), n + 1),
                xx,
                30,
                FONT_HELP,
                false,
            ));
            els.push(Element::text(
                self.team_names[n].clone(),
                xx,
                42,
                FONT_DEFAULT,
                false,
            ));
        }

        // WaitForKey3(305,180,ch) + getch(306,180,243)
        hud::push_wait_for_key(
            &mut els,
            &self.resources.langbase,
            305,
            180,
            BG_TEAMCUP,
            FONT_DEFAULT,
            FONT_DEFAULT,
            self.cursor_visible,
        );

        els
    }

    fn showteams_elements(&self) -> Vec<Element> {
        let mut els = new_screen_with_bg(1, BG_TEAMCUP);

        // Title
        els.push(Element::text(
            self.resources.langbase.lstr(111).to_string(),
            30,
            6,
            FONT_DEFAULT,
            false,
        ));

        // Team grid: 3 columns, 5 rows
        let mut x = 5i32;
        let mut y = 24i32;
        self.store.try_with_team_cup(|tc| {
            for &team_idx in tc.team_order.iter().rev() {
                let team = &tc.teams[team_idx];
                let is_human = team.is_human_team;

                // Team name in white
                els.push(Element::text(
                    shorten_name(&team.name, &self.resources.font, 95),
                    x,
                    y,
                    FONT_DEFAULT,
                    false,
                ));

                // Jumper names: gold if human, gray if AI
                let jcolor = if is_human { FONT_GOLD } else { FONT_HELP };
                for (j, member) in team.members.iter().enumerate() {
                    // Pascal: t2=1..4 → y+1+(t2*6)
                    els.push(Element::text(
                        shorten_name(&member.competitor.name, &self.resources.font, 90),
                        x + 4,
                        y + 7 + j as i32 * 6,
                        jcolor,
                        false,
                    ));
                }

                x += 102;
                if x > 240 {
                    x = 5;
                    y += 35;
                }
            }
        });

        // WaitForKey3 at top right (305,6)
        hud::push_wait_for_key(
            &mut els,
            &self.resources.langbase,
            305,
            6,
            BG_TEAMCUP,
            FONT_DEFAULT,
            FONT_DEFAULT,
            self.cursor_visible,
        );

        els
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
        match self.controller.drive_competition(&scene) {
            Ok(Some(cmd)) => self.apply_command(cmd),
            Ok(None) => {
                self.ui_state.enter_error("No competition running".into());
            }
            Err(e) => {
                self.ui_state.enter_error(e.to_string());
            }
        }
    }

    fn apply_command(&mut self, command: TeamCupUiCommand) {
        match command {
            TeamCupUiCommand::HumanJump(req) => {
                let phase_label = if req.context.round_idx == 0 {
                    self.resources.langbase.lstr(54).to_string()
                } else {
                    self.resources.langbase.lstr(55).to_string()
                };
                handle_human_jump(
                    &mut self.scene,
                    &self.ui_state,
                    &self.resources,
                    &self.store,
                    req.participant,
                    req.hill_idx,
                    phase_label,
                    Some(req.context.team_name.clone()),
                );
                self.ui_state.enter_jump();
            }
            TeamCupUiCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.ui_state.enter_results();
            }
            TeamCupUiCommand::Done => {
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
            self.store.try_with_team_cup_mut(|tc| {
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

        if self.ui_state.is_result_acknowledged()
            && !self.ui_state.is_outcome_recorded()
            && self
                .scene
                .as_ref()
                .is_some_and(|s| self.controller.record_finished_human_jump(s))
        {
            self.ui_state.mark_outcome_recorded();
        }

        // Drive competition only after human jump outcome is recorded,
        // not every frame during the jump (avoids recreating the scene).
        if self.ui_state.is_outcome_recorded() && self.ui_state.render_mode() != RenderMode::Results
        {
            if let Some(ref scene) = self.scene {
                match self.controller.drive_competition(scene) {
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
                    scene.elements()
                } else {
                    vec![]
                }
            }
            RenderMode::Results => {
                let mut els = new_screen_with_bg(1, BG_TEAMCUP);
                let standings_kind = match self.results_kind {
                    TeamCupResultsKind::Standings => TeamCupStandingsKind::Overall,
                    TeamCupResultsKind::LegResults => TeamCupStandingsKind::Leg,
                };
                let (header, standings) = self
                    .store
                    .try_with_team_cup(|tc| {
                        let standings = tc.standings_runtime(standings_kind);
                        let leg = tc.current_leg + 1;
                        let round = tc.current_round + 1;
                        let jumper = tc.current_jumper_slot + 1;
                        let header = match self.results_kind {
                            TeamCupResultsKind::LegResults => format!(
                                "{} {} {} 6 - R {} - {} {}",
                                self.resources.langbase.lstr(81),
                                leg,
                                self.resources.langbase.lstr(8),
                                round,
                                self.resources.langbase.lstr(88),
                                jumper,
                            ),
                            TeamCupResultsKind::Standings => {
                                let text = self.resources.langbase.lstr(91);
                                format!("{} {} 6", text, leg)
                            }
                        };
                        (header, standings)
                    })
                    .unwrap_or_default();

                els.push(Element::text(header, 30, 6, FONT_DEFAULT, false));

                let mut last_rank = 0usize;
                let mut y = 23i32;
                for (i, entry) in standings.iter().enumerate() {
                    if i >= 15 {
                        break;
                    }
                    let is_human = entry.is_human;

                    // Pascal Entry: rank only when different from last
                    if entry.rank != last_rank && entry.rank > 0 {
                        let c = if is_human { FONT_GOLD } else { FONT_HELP };
                        els.push(Element::text(format!("{}.", entry.rank), 24, y, c, true));
                    }
                    last_rank = entry.rank;

                    // Name: column 2 = 32, shortened to 122px
                    let c = if is_human { FONT_GOLD } else { FONT_HELP };
                    els.push(Element::text(
                        shorten_name(&entry.name, &self.resources.font, 122),
                        32,
                        y,
                        c,
                        false,
                    ));

                    // Points: column 3 = 184
                    els.push(Element::right_text(
                        entry.primary_score.to_string(),
                        184,
                        y,
                        c,
                    ));

                    y += 10;
                }

                // Pascal WaitForKey: Done-) at bottom right
                els.push(Element::right_text(
                    format!("{}-)", self.resources.langbase.lstr(248)),
                    319,
                    13,
                    FONT_HELP,
                ));
                els
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
        if self.ui_state.render_mode() == RenderMode::Error {
            if matches!(event, Event::Keyboard(_)) {
                return Some(RouteTarget::Back);
            }
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
            if scene.outcome().is_some() && !self.ui_state.is_result_acknowledged() {
                if matches!(event, Event::Keyboard(_)) {
                    self.ui_state.acknowledge_outcome();
                }
                return None;
            }
        }

        if self.ui_state.render_mode() == RenderMode::Results {
            if matches!(event, Event::Keyboard(_)) {
                self.ui_state.dismiss_results();
                self.store.try_with_team_cup_mut(|tc| {
                    tc.advance_results_runtime(TeamCupResultsKind::LegResults);
                });
                self.drive_until_visible();
            }
            return None;
        }

        // Forward keyboard to jump controls while jump is active
        if let Some(ref scene) = self.scene {
            if self.phase == ViewPhase::Jumping {
                let mut session = scene.session_mut();
                let action = JumpInputController.handle_event(event, &mut session);
                if matches!(action, JumpInputAction::RouteBack) {
                    return Some(RouteTarget::Back);
                }
            }
        }

        None
    }
}

// Shared helpers -----------------------------------------------------------

fn drop_team_cup_header(els: &mut Vec<Element>, resources: &ResourcesRef, store: &StoreRef) {
    // Title: "Get Ready for the Team Cup" (lstr 111)
    els.push(Element::text(
        resources.langbase.lstr(111).to_string(),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));

    // Schedule header (lstr 112)
    els.push(Element::text(
        resources.langbase.lstr(112).to_string(),
        30,
        110,
        FONT_DEFAULT,
        false,
    ));

    // 6 hill names
    if let Some(schedule) = store.try_with_team_cup(|tc| tc.schedule.clone()) {
        for (i, &hill_idx) in schedule.iter().enumerate() {
            let hill_name = resources
                .hills
                .hill(hill_idx)
                .map(|h| format!("{}. {} K{}", i + 1, h.name, h.kr))
                .unwrap_or_else(|| format!("{}. Hill {}", i + 1, hill_idx));
            els.push(Element::text(
                hill_name,
                30,
                124 + i as i32 * 10,
                FONT_GOLD,
                false,
            ));
        }
    }
}

fn jumper_names_els(els: &mut Vec<Element>, store: &StoreRef, team_n: usize, xx: i32) {
    let jumpers: Vec<String> = store
        .try_with_team_cup(|tc| {
            tc.teams
                .iter()
                .filter(|t| t.is_human_team)
                .nth(team_n)
                .map(|t| {
                    t.members
                        .iter()
                        .map(|m| m.competitor.name.clone())
                        .collect()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    for (j, jname) in jumpers.iter().enumerate() {
        els.push(Element::text(
            jname.clone(),
            xx + 13,
            56 + j as i32 * 10,
            FONT_GOLD,
            false,
        ));
    }
}
