use std::cell::RefCell;

use crate::competition::koth::types::{KothParticipant, KothRuntime};
use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;
use crate::competition::team_cup::types::TeamCupStandingsKind;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};
use crate::competition::ActiveCompetition;
use crate::gfx::sprites::Sprite;
use crate::gfx::theme::{FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::jump::hud;
use crate::jump::types::{JumpOutcome, JumpPhase, JumpTelemetry};
use crate::rng::Random;
use crate::store::{GameState, ResourcesRef};
use crate::text::format::format_decimal;
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;
use crate::views::jump::competition::ui_state::CompetitionUiState;

#[derive(Debug, Clone)]
pub struct KothOverlayInfo {
    pub alive_count: usize,
    pub total_count: usize,
    pub last_name: String,
    pub last_points: f64,
    pub jump_round: u8,
    pub jump_rounds_per_elimination: u8,
}

#[derive(Debug, Clone)]
pub struct OverlayData {
    pub phase: CompetitionPhase,
    pub style: CupStyle,
    pub current_event: usize,
    pub current_hill: usize,
    pub current_participant: Option<Participant>,
    pub event_standings_top5: Vec<EventStandingEntry>,
    pub wc_standings_top5: Vec<WcStandingEntry>,
    pub coach_style: u8,
    pub koth_info: Option<KothOverlayInfo>,
    pub final_result: Option<FinalResultInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalResultInfo {
    pub team_name: Option<String>,
    pub placement: usize,
    pub current_distance: f64,
    pub current_score: f64,
    pub total_score: f64,
    pub round_one_distance: Option<f64>,
    pub injury: u8,
}

#[derive(Debug, Clone)]
pub struct EventStandingEntry {
    pub name: String,
    pub points: f64,
}

#[derive(Debug, Clone)]
pub struct WcStandingEntry {
    pub name: String,
    pub points: i32,
}

impl OverlayData {
    pub fn collect(state: &GameState) -> Option<Self> {
        state
            .active_competition
            .as_ref()
            .and_then(|active| match active {
                ActiveCompetition::Training => None,
                ActiveCompetition::Individual(comp) => Some(Self::from_individual(comp, state)),
                ActiveCompetition::TeamCup(comp) => Some(Self::from_team_cup(comp, state)),
                ActiveCompetition::Koth(comp) => Some(Self::from_koth(comp, state)),
            })
    }

    fn from_individual(c: &Competition, state: &GameState) -> Self {
        let event_standings = c.event_standings();
        let event_top5 = event_standings
            .iter()
            .take(5)
            .filter_map(|p| {
                p.points
                    .filter(|&pts| pts > 0.0)
                    .map(|pts| EventStandingEntry {
                        name: p.display_name().to_string(),
                        points: pts,
                    })
            })
            .collect();

        let wc_standings = c.overall_standings();
        let wc_top5 = wc_standings
            .iter()
            .take(5)
            .filter(|p| p.wc_points > 0)
            .map(|p| WcStandingEntry {
                name: p.display_name().to_string(),
                points: p.wc_points,
            })
            .collect();

        Self {
            phase: c.phase(),
            style: c.style(),
            current_event: c.current_event,
            current_hill: c.current_hill(),
            current_participant: c.current_jumper().map(|idx| c.participant(idx).clone()),
            event_standings_top5: event_top5,
            wc_standings_top5: wc_top5,
            coach_style: coach_style_for_profile(
                state,
                c.current_jumper()
                    .and_then(|idx| c.participant(idx).profile_idx),
            ),
            koth_info: None,
            final_result: None,
        }
    }

    fn from_koth(c: &KothRuntime, state: &GameState) -> Self {
        let alive_count = c.participants.iter().filter(|p| p.is_alive()).count();
        let total_count = c.participants.len();

        let last_place = koth_last_place(c);
        let (last_name, last_points) = last_place
            .map(|p| (p.competitor.name.clone(), p.total_points))
            .unwrap_or_default();
        Self {
            phase: CompetitionPhase::Round1,
            style: CupStyle::CustomCup,
            current_event: c.current_elimination_round as usize,
            current_hill: c.hill_idx,
            current_participant: c
                .participants
                .get(c.current_participant_pos)
                .map(|p| Participant::from_competitor(p.competitor.clone())),
            event_standings_top5: Vec::new(),
            wc_standings_top5: Vec::new(),
            coach_style: coach_style_for_profile(
                state,
                c.participants
                    .get(c.current_participant_pos)
                    .and_then(|p| p.competitor.profile_idx),
            ),
            koth_info: Some(KothOverlayInfo {
                alive_count,
                total_count,
                last_name,
                last_points,
                jump_round: c.current_jump_round,
                jump_rounds_per_elimination: c.jump_rounds_per_elimination,
            }),
            final_result: None,
        }
    }

    fn from_team_cup(tc: &TeamCupRuntime, state: &GameState) -> Self {
        let leg_standings = tc.standings(TeamCupStandingsKind::Leg);
        let hill_idx = tc.current_hill_idx();
        let event_top5 = leg_standings
            .iter()
            .take(5)
            .filter(|e| e.primary_score > 0.0)
            .map(|e| EventStandingEntry {
                name: e.name.clone(),
                points: e.primary_score,
            })
            .collect();
        Self {
            phase: CompetitionPhase::Round1,
            style: CupStyle::TeamCup,
            current_event: tc.current_leg,
            current_hill: hill_idx,
            current_participant: tc
                .team_order
                .get(tc.current_team_order_pos)
                .and_then(|&team_idx| tc.teams.get(team_idx))
                .and_then(|team| team.members.get(tc.current_jumper_slot))
                .map(|member| Participant::from_competitor(member.competitor.clone())),
            event_standings_top5: event_top5,
            wc_standings_top5: Vec::new(),
            coach_style: coach_style_for_profile(
                state,
                tc.team_order
                    .get(tc.current_team_order_pos)
                    .and_then(|&team_idx| tc.teams.get(team_idx))
                    .and_then(|team| team.members.get(tc.current_jumper_slot))
                    .and_then(|member| member.competitor.profile_idx),
            ),
            koth_info: None,
            final_result: None,
        }
    }

    fn with_final_result(
        mut self,
        state: &GameState,
        participant_id: usize,
        team_name: &str,
        outcome: Option<JumpOutcome>,
    ) -> Self {
        self.final_result = outcome
            .and_then(|outcome| final_result_info(state, participant_id, team_name, outcome));
        self
    }
}

fn placement_for_score(score: f64, others: &[f64]) -> usize {
    others.iter().filter(|&&other| other > score).count() + 1
}

fn projected_result(
    team_name: Option<String>,
    current_distance: f64,
    current_score: f64,
    prior_score: f64,
    round_one_distance: Option<f64>,
    injury: u8,
    other_scores: &[f64],
) -> FinalResultInfo {
    let total_score = prior_score + current_score;
    FinalResultInfo {
        team_name,
        placement: placement_for_score(total_score, other_scores),
        current_distance,
        current_score,
        total_score,
        round_one_distance,
        injury,
    }
}

fn should_render_final_result(phase: JumpPhase, result: Option<&FinalResultInfo>) -> bool {
    phase == JumpPhase::Result && result.is_some()
}

fn final_result_info(
    state: &GameState,
    participant_id: usize,
    team_name: &str,
    outcome: JumpOutcome,
) -> Option<FinalResultInfo> {
    let active = state.active_competition.as_ref()?;
    let team_name = (!team_name.is_empty()).then(|| team_name.to_string());
    match active {
        ActiveCompetition::Training => None,
        ActiveCompetition::Individual(c) => {
            if matches!(c.phase(), CompetitionPhase::Training(_)) {
                return None;
            }
            let participant = c
                .event_standings()
                .into_iter()
                .find(|p| p.id == participant_id)?;
            let round_one_distance =
                (c.phase() == CompetitionPhase::Round2).then_some(participant.round1_len);
            let prior_score = if round_one_distance.is_some() {
                participant.round1_score
            } else {
                0.0
            };
            let other_scores: Vec<_> = c
                .event_standings()
                .into_iter()
                .filter(|p| p.id != participant_id)
                .filter_map(|p| p.points)
                .collect();
            Some(projected_result(
                team_name,
                outcome.distance,
                outcome.score,
                prior_score,
                round_one_distance,
                outcome.injury,
                &other_scores,
            ))
        }
        ActiveCompetition::TeamCup(tc) => {
            let context = tc.current_jump_context();
            let team = tc.teams.get(context.team_idx)?;
            let other_scores: Vec<_> = tc
                .teams
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != context.team_idx)
                .map(|(_, team)| team.leg_score)
                .collect();
            let round_one_distance = (context.round_idx == 1).then(|| {
                team.members
                    .get(context.member_idx)
                    .and_then(|member| {
                        member
                            .jumps
                            .iter()
                            .rev()
                            .find(|jump| jump.leg == context.leg_idx && jump.round == 0)
                    })
                    .map_or(0.0, |jump| jump.distance)
            });
            Some(projected_result(
                team_name.or_else(|| Some(team.name.clone())),
                outcome.distance,
                outcome.score,
                team.leg_score,
                round_one_distance,
                outcome.injury,
                &other_scores,
            ))
        }
        ActiveCompetition::Koth(koth) => {
            let participant = koth
                .participants
                .iter()
                .find(|participant| participant.competitor.id == participant_id)?;
            let other_scores: Vec<_> = koth
                .participants
                .iter()
                .enumerate()
                .filter(|(_, participant)| {
                    participant.competitor.id != participant_id && participant.is_alive()
                })
                .map(|(_, participant)| participant.total_points)
                .collect();
            Some(projected_result(
                team_name,
                outcome.distance,
                outcome.score,
                participant.total_points,
                None,
                outcome.injury,
                &other_scores,
            ))
        }
    }
}

fn coach_style_for_profile(state: &GameState, profile_idx: Option<usize>) -> u8 {
    profile_idx
        .and_then(|idx| state.profiles.profiles.get(idx))
        .map_or(0, |profile| profile.coach_style as u8)
}

const fn coach_uses_low_grade_range(grade: u8) -> bool {
    grade <= 3
}

fn coach_is_result_phase(phase: JumpPhase) -> bool {
    phase == JumpPhase::Result
}

pub struct OverlayContext {
    pub kind: OverlayKind,
    pub participant: Participant,
    pub hill_idx: usize,
    pub frame_counter: i32,
    pub data: OverlayData,
    pub telemetry: Option<JumpTelemetry>,
    pub scene_phase: JumpPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    None,
    Keymap,
    CyclingWithInfoBox,
    Round2WithInfoBox,
    Coach,
    Koth,
}

pub struct CompetitionOverlay {
    resources: ResourcesRef,
    coach_rng: RefCell<Random>,
    coach_message: RefCell<Option<(JumpTelemetry, u8, String)>>,
}

pub struct SceneOverlaySnapshot<'a> {
    pub scene_phase: Option<JumpPhase>,
    pub frame_counter: i32,
    pub telemetry: Option<JumpTelemetry>,
    pub outcome: Option<JumpOutcome>,
    pub participant_id: usize,
    pub team_name: &'a str,
}

fn wrap_coach_text(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::with_capacity(32);
    for ch in text.chars() {
        line.push(if ch == '*' { ' ' } else { ch });

        if (line.chars().count() >= 30 && ch == ' ') || (ch == '*' && line.chars().count() >= 15) {
            if ch == '*' {
                line.pop();
            }
            lines.push(std::mem::take(&mut line));
        }
    }
    lines.push(line);
    lines
}

impl CompetitionOverlay {
    pub fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            coach_rng: RefCell::new(Random::default()),
            coach_message: RefCell::new(None),
        }
    }

    pub fn context(
        &self,
        ui_state: &CompetitionUiState,
        state: &GameState,
        snapshot: SceneOverlaySnapshot<'_>,
    ) -> Option<OverlayContext> {
        let SceneOverlaySnapshot {
            scene_phase,
            frame_counter,
            telemetry,
            outcome,
            participant_id,
            team_name,
        } = snapshot;
        let scene_phase = scene_phase?;
        let data = OverlayData::collect(state)?.with_final_result(
            state,
            participant_id,
            team_name,
            outcome,
        );
        let kind = self.resolve_kind(&data, scene_phase, ui_state, &telemetry);
        let participant = data
            .current_participant
            .clone()
            .unwrap_or_else(|| Participant::computer(0, 0, String::new()));
        Some(OverlayContext {
            kind,
            participant,
            hill_idx: data.current_hill,
            frame_counter,
            data,
            telemetry,
            scene_phase,
        })
    }

    fn resolve_kind(
        &self,
        data: &OverlayData,
        scene_phase: JumpPhase,
        ui_state: &CompetitionUiState,
        telemetry: &Option<JumpTelemetry>,
    ) -> OverlayKind {
        if scene_phase == JumpPhase::Disqualified {
            return OverlayKind::None;
        }

        if let Some(t) = telemetry {
            if t.grade > 0
                && data.coach_style > 0
                && data
                    .current_participant
                    .as_ref()
                    .is_some_and(|p| !p.is_computer)
                && coach_is_result_phase(scene_phase)
            {
                return OverlayKind::Coach;
            }
        }

        if data.koth_info.is_some() && matches!(scene_phase, JumpPhase::Info | JumpPhase::Result) {
            return OverlayKind::Koth;
        }

        let first_event = data.current_event == 0;
        let show_keymap = ui_state.is_first_human_onbar()
            && first_event
            && data
                .current_participant
                .as_ref()
                .is_some_and(|p| !p.is_computer);

        match (data.phase, scene_phase) {
            (CompetitionPhase::Round2, JumpPhase::Info) if data.style != CupStyle::CustomCup => {
                OverlayKind::Round2WithInfoBox
            }
            (CompetitionPhase::Qualification, JumpPhase::OnBar) if show_keymap => {
                OverlayKind::Keymap
            }

            (CompetitionPhase::Qualification | CompetitionPhase::Round1, JumpPhase::Info) => {
                OverlayKind::CyclingWithInfoBox
            }
            _ => OverlayKind::None,
        }
    }

    pub fn render(&self, cx: &mut dyn UiCanvas, ctx: &OverlayContext, state: &GameState) {
        match ctx.kind {
            OverlayKind::None => {}
            OverlayKind::Keymap => hud::push_keymap(cx, &self.resources.langbase, &state.config),
            OverlayKind::CyclingWithInfoBox => {
                self.cycling_info_elements(cx, ctx.frame_counter, ctx.hill_idx, &ctx.data, state);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(cx, state, &ctx.participant, false);
                }
            }
            OverlayKind::Round2WithInfoBox => {
                self.cycling_info_elements(cx, ctx.frame_counter, ctx.hill_idx, &ctx.data, state);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(cx, state, &ctx.participant, true);
                }
            }
            OverlayKind::Coach => {
                if let Some(telemetry) = ctx.telemetry {
                    self.render_coach_feedback(cx, telemetry, ctx.data.coach_style);
                }
            }
            OverlayKind::Koth => {
                if ctx.scene_phase == JumpPhase::Info {
                    let phase = (ctx.frame_counter as usize) % 292;
                    if phase <= 130 {
                        self.koth_info_elements(cx, ctx);
                    } else if (146..=276).contains(&phase) {
                        self.hill_info_elements(cx, state, ctx.hill_idx);
                    }
                } else if ctx.scene_phase == JumpPhase::Result {
                    self.final_result_elements(cx, ctx);
                }
            }
        }
        if ctx.kind != OverlayKind::Koth
            && should_render_final_result(ctx.scene_phase, ctx.data.final_result.as_ref())
        {
            self.final_result_elements(cx, ctx);
        }
    }

    fn final_result_elements(&self, cx: &mut dyn UiCanvas, ctx: &OverlayContext) {
        let Some(result) = ctx.data.final_result.as_ref() else {
            return;
        };
        cx.right_text((255, 45), FONT_TEAL, &format!("({}.)", result.placement));
        if let Some(round_one) = result.round_one_distance {
            cx.right_text((255, 33), FONT_TEAL, &format!("{round_one:.1}m"));
        }
        cx.right_text(
            (311, 55),
            FONT_TEAL,
            &format!("({})", format_decimal(result.current_score)),
        );
        if result.injury > 0 && ctx.data.style == CupStyle::WorldCup {
            let lang = &self.resources.langbase;
            let missed = result.injury.saturating_sub(1);
            let injury_text = match result.injury {
                1 => format!("{} {}", lang.tr(75), lang.tr(77)),
                2 => format!("{} {}", lang.tr(75), lang.tr(78)),
                _ => format!("{} {} {}", lang.tr(75), missed, lang.tr(76)),
            };
            cx.right_text((308, 64), FONT_GOLD, &injury_text);
        }
    }

    pub(crate) fn render_coach_feedback(&self, cx: &mut dyn UiCanvas, t: JumpTelemetry, style: u8) {
        if style == 0 || t.grade == 0 {
            return;
        }
        let base = 360 + style as usize * 40;
        let lang = &self.resources.langbase;

        let text = {
            let cached = self.coach_message.borrow();
            cached
                .as_ref()
                .filter(|(telemetry, cached_style, _)| *telemetry == t && *cached_style == style)
                .map(|(_, _, text)| text.clone())
        };
        let text = text.unwrap_or_else(|| {
            let cstr0 = self.coach_range(lang, base + 2, t.body_angle, &[49, 61, 200]);
            let cstr1 = if coach_uses_low_grade_range(t.grade) {
                self.coach_range(lang, base + 5, t.grade, &[1, 2, 3])
            } else {
                self.coach_range(lang, base + 10, t.grade / 10, &[5, 8, 9, 10, 11, 20])
            };
            let cstr2 = self.coach_range(
                lang,
                base + 18,
                t.takeoff_timing,
                &[5, 9, 12, 15, 16, 19, 23, 50],
            );
            let mut cstr3 =
                self.coach_range(lang, base + 28, t.height, &[49, 55, 60, 64, 70, 90, 200]);

            if t.grade == 1 {
                cstr3 = lang.tr(base + 35).to_string();
            }

            let cstr0 = if t.grade < 10 { cstr1.clone() } else { cstr0 };

            let mut rng = self.coach_rng.borrow_mut();
            let pick_a = if rng.random_i32(2) == 0 {
                &cstr0
            } else {
                &cstr1
            };
            let pick_b = if rng.random_i32(2) == 0 {
                &cstr2
            } else {
                &cstr3
            };
            let text = format!("{pick_a}*{pick_b}");
            *self.coach_message.borrow_mut() = Some((t, style, text.clone()));
            text
        });

        cx.sprite(Sprite::JumperInfoBox as u16, (3, 150));
        cx.text((12, 150), FONT_TEAL, lang.tr(400));
        cx.text((12, 160), FONT_TEAL, "\"");

        let mut y = 152i32;
        let lines = wrap_coach_text(&text);
        let last = lines.len().saturating_sub(1);
        for (index, line) in lines.iter().enumerate() {
            if index < last {
                if y < 190 {
                    y += 8;
                }
                cx.text((18, y), FONT_TEAL, line);
            } else if !line.is_empty() {
                if line.chars().count() < 2 {
                    cx.text((18, y), FONT_TEAL, &format!("{line}\""));
                } else {
                    if y < 192 {
                        y += 8;
                    }
                    cx.text((18, y), FONT_TEAL, &format!("{line}\""));
                }
            }
        }
    }

    fn coach_range(&self, lang: &LangBase, base: usize, val: u8, thresholds: &[u8]) -> String {
        let idx = thresholds
            .iter()
            .position(|&t| val <= t)
            .unwrap_or(thresholds.len());
        lang.tr(base + idx).to_string()
    }

    fn jumper_info_box(
        &self,
        cx: &mut dyn UiCanvas,
        state: &GameState,
        participant: &Participant,
        round2_with_r1: bool,
    ) {
        let lang = &self.resources.langbase;
        let (phase, rank, quali_wc) = state
            .active_competition
            .as_ref()
            .and_then(|active| {
                let c = active.individual()?;
                let phase = c.phase();
                let rank = if round2_with_r1 {
                    participant.round1_rank
                } else {
                    let standings = c.event_standings();
                    standings
                        .iter()
                        .position(|p| p.id == participant.id)
                        .map_or(0, |i| i + 1)
                };
                let quali_wc = phase == CompetitionPhase::Qualification
                    && matches!(participant.qual, QualificationStatus::PreQualified);
                Some((phase, rank, quali_wc))
            })
            .unwrap_or((CompetitionPhase::Qualification, 0, false));

        let phase_label = match phase {
            CompetitionPhase::Training(n) => format!("{} {}", lang.tr(52), n),
            CompetitionPhase::Qualification => lang.tr(53).to_string(),
            CompetitionPhase::Round1 => lang.tr(54).to_string(),
            CompetitionPhase::Round2 => lang.tr(55).to_string(),
            _ => lang.tr(51).to_string(),
        };

        let name = if quali_wc {
            format!("{} Q WC", participant.display_name())
        } else if round2_with_r1 && rank > 0 {
            format!("{} ({}.)", participant.display_name(), rank)
        } else {
            participant.display_name().to_string()
        };

        let r1text = if round2_with_r1 {
            Some(format!(
                "{} ({}µ)",
                format_decimal(participant.round1_score),
                format_decimal(participant.round1_len)
            ))
        } else {
            None
        };
        hud::push_jumper_info_box(
            cx,
            &self.resources.font,
            &self.resources.langbase,
            &phase_label,
            &name,
            r1text.as_deref().map(|text| (text, FONT_GRAY)),
        );
    }

    fn cycling_info_elements(
        &self,
        cx: &mut dyn UiCanvas,
        frame_counter: i32,
        hill_idx: usize,
        data: &OverlayData,
        state: &GameState,
    ) {
        let has_wc_leader = data.wc_standings_top5.first().is_some_and(|e| e.points > 0);
        let has_event_leader = data
            .event_standings_top5
            .first()
            .is_some_and(|e| e.points > 0.0);

        if !has_event_leader {
            if has_wc_leader {
                let phase = (frame_counter as usize) % 292;
                if phase <= 130 {
                    self.hill_info_elements(cx, state, hill_idx);
                } else if (146..=276).contains(&phase) {
                    self.wc_standings_elements(cx, data, state);
                } else {
                    hud::push_info_panel_frame(cx);
                }
            } else {
                self.hill_info_elements(cx, state, hill_idx);
            }
            return;
        }

        let cycle = if has_wc_leader { 438 } else { 292 };
        let phase = (frame_counter as usize) % cycle;

        if phase <= 130 {
            self.top5_event_elements(cx, data, state);
        } else if (146..=276).contains(&phase) {
            self.hill_info_elements(cx, state, hill_idx);
        } else if has_wc_leader && (292..=422).contains(&phase) {
            self.wc_standings_elements(cx, data, state);
        } else {
            hud::push_info_panel_frame(cx);
        }
    }

    fn top5_event_elements(&self, cx: &mut dyn UiCanvas, data: &OverlayData, state: &GameState) {
        let lang = &self.resources.langbase;
        hud::push_info_panel_frame(cx);
        let hill_name_k = self
            .resources
            .hills
            .hill(data.current_hill)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        cx.right_text((308, 9), FONT_GOLD, &hill_name_k);

        for (i, entry) in data.event_standings_top5.iter().enumerate() {
            if entry.points > 0.0 {
                cx.right_text(
                    (308, 20 + i as i32 * 7),
                    FONT_GOLD,
                    &format!("{}  {}", entry.name, format_decimal(entry.points)),
                );
            }
        }

        if state.config.event_gap != 0 {
            if let Some(ref pel) = data.current_participant {
                let leader_pts = data.event_standings_top5.first().map_or(0.0, |e| e.points);
                let current_pts = pel.points.unwrap_or(0.0);
                let temp = leader_pts - current_pts;
                if temp > 0.0 {
                    let label = lang.tr(62);
                    cx.right_text(
                        (308, 62),
                        FONT_GOLD,
                        &format!("{}: {}", label, format_decimal(temp + 0.1)),
                    );
                }
            }
        }
    }

    fn hill_info_elements(&self, cx: &mut dyn UiCanvas, state: &GameState, hill_idx: usize) {
        let hill = self.resources.hills.hill(hill_idx);
        let hill_name_k = hill
            .as_ref()
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let record = hill
            .as_ref()
            .and_then(|h| state.records.hill_record(&h.record_key));
        let goal = if state.config.goals_enabled != 0 {
            hill.as_ref()
                .and_then(|h| state.records.hill_goal(&h.record_key))
                .copied()
        } else {
            None
        };
        hud::push_hill_record_info(cx, &self.resources.langbase, &hill_name_k, record, goal);
    }

    fn koth_info_elements(&self, cx: &mut dyn UiCanvas, ctx: &OverlayContext) {
        let lang = &self.resources.langbase;
        let Some(ref ki) = ctx.data.koth_info else {
            return;
        };
        let total = ki.total_count;
        let left = ki.alive_count;
        hud::push_info_panel_frame(cx);

        let str1 = format!("{} {} {}", lang.tr(67), left, lang.tr(8));
        cx.right_text((308, 9), FONT_GOLD, &format!("{str1} {total}"));

        if !ki.last_name.is_empty() {
            let label = if ki.jump_round == 0 && ki.jump_rounds_per_elimination > 1 {
                lang.tr(69)
            } else {
                lang.tr(68)
            };
            cx.right_text((308, 19), FONT_GOLD, label);
            let pts_str = format_decimal(ki.last_points);
            cx.right_text(
                (308, 29),
                FONT_GOLD,
                &format!("{} ${}", ki.last_name, pts_str),
            );
        }
    }

    fn wc_standings_elements(&self, cx: &mut dyn UiCanvas, data: &OverlayData, state: &GameState) {
        let lang = &self.resources.langbase;
        hud::push_info_panel_frame(cx);
        cx.right_text((308, 9), FONT_GOLD, lang.tr(70));
        for (i, entry) in data.wc_standings_top5.iter().enumerate() {
            let s = format!("{}  {}", entry.name, entry.points);
            cx.right_text((308, 20 + i as i32 * 7), FONT_GOLD, &s);
        }
        if state.config.wc_gap != 0 {
            if let (Some(leader), Some(current)) = (
                data.wc_standings_top5.first(),
                data.current_participant.as_ref(),
            ) {
                let diff = leader.points - current.wc_points;
                if diff > 0 {
                    cx.right_text((308, 62), FONT_GOLD, &format!("{}: {diff}", lang.tr(62)));
                }
            }
        }
    }
}

fn koth_last_place(c: &KothRuntime) -> Option<&KothParticipant> {
    c.participants
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            p.is_alive()
                && !c.human_indices.contains(i)
                && p.jumps.iter().any(|jump| {
                    jump.elimination_round == c.current_elimination_round
                        && jump.jump_round == c.current_jump_round
                })
        })
        .min_by(|(_, a), (_, b)| a.total_points.total_cmp(&b.total_points))
        .map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::{
        coach_is_result_phase, coach_style_for_profile, coach_uses_low_grade_range,
        koth_last_place, projected_result, should_render_final_result, wrap_coach_text,
    };
    use crate::competition::core::competitor::Competitor;
    use crate::competition::koth::types::{
        KothJumpResult, KothParticipant, KothPhase, KothRuntime,
    };
    use crate::data::profile::Profile;
    use crate::jump::types::JumpPhase;
    use crate::rng::Random;
    use crate::store::GameState;

    #[test]
    fn round_two_projection_keeps_current_total_distance_team_and_injury() {
        let result = projected_result(
            Some("TEAM A".to_string()),
            125.0,
            118.5,
            121.0,
            Some(104.5),
            3,
            &[250.0, 238.0, 200.0],
        );

        assert_eq!(result.team_name.as_deref(), Some("TEAM A"));
        assert_eq!(result.placement, 2);
        assert_eq!(result.current_distance, 125.0);
        assert_eq!(result.current_score, 118.5);
        assert_eq!(result.total_score, 239.5);
        assert_eq!(result.round_one_distance, Some(104.5));
        assert_eq!(result.injury, 3);
    }

    #[test]
    fn projected_placement_excludes_current_jumper_pre_jump_score() {
        let result = projected_result(None, 120.0, 110.0, 0.0, None, 0, &[150.0, 105.0, 90.0]);

        assert_eq!(result.placement, 2);
    }

    #[test]
    fn koth_result_phase_has_renderable_final_overlay_content() {
        let result = projected_result(None, 100.0, 100.0, 50.0, None, 0, &[140.0]);

        assert!(should_render_final_result(JumpPhase::Result, Some(&result)));
        assert!(!should_render_final_result(JumpPhase::Info, Some(&result)));
    }

    #[test]
    fn koth_last_place_uses_only_participants_handled_in_current_attempt() {
        let jump = |round: u8, score: f64| KothJumpResult {
            elimination_round: 2,
            jump_round: round,
            distance: 100.0,
            score,
        };
        let participants = vec![
            KothParticipant {
                competitor: Competitor::computer(0, 0, "Human".to_string(), None),
                total_points: 1.0,
                eliminated_in_round: u8::MAX,
                jumps: vec![jump(1, 1.0)],
            },
            KothParticipant {
                competitor: Competitor::computer(1, 1, "Handled".to_string(), None),
                total_points: 40.0,
                eliminated_in_round: u8::MAX,
                jumps: vec![jump(1, 40.0)],
            },
            KothParticipant {
                competitor: Competitor::computer(2, 2, "Not handled".to_string(), None),
                total_points: 0.0,
                eliminated_in_round: u8::MAX,
                jumps: vec![],
            },
        ];
        let koth = KothRuntime {
            participants,
            human_indices: vec![0],
            hill_idx: 0,
            jump_rounds_per_elimination: 2,
            phase: KothPhase::Jumping,
            current_elimination_round: 2,
            current_jump_round: 1,
            current_participant_pos: 2,
            rng: Random::default(),
        };

        assert_eq!(koth_last_place(&koth).unwrap().competitor.name, "Handled");
    }

    #[test]
    fn coach_style_uses_current_profile_instead_of_first_active_profile() {
        let mut state = GameState::default();
        state.profiles.profiles = vec![
            Profile {
                coach_style: 1,
                ..Profile::default()
            },
            Profile {
                coach_style: 4,
                ..Profile::default()
            },
        ];
        state.profiles.active_order = vec![0, 1];

        assert_eq!(coach_style_for_profile(&state, Some(1)), 4);
        assert_eq!(coach_style_for_profile(&state, None), 0);
    }

    #[test]
    fn coach_grade_four_uses_the_pascal_tens_range() {
        assert!(!coach_uses_low_grade_range(4));
        assert!(coach_uses_low_grade_range(3));
    }

    #[test]
    fn coach_is_only_available_after_result() {
        assert!(!coach_is_result_phase(JumpPhase::Info));
        assert!(!coach_is_result_phase(JumpPhase::OnBar));
        assert!(coach_is_result_phase(JumpPhase::Result));
    }

    #[test]
    fn coach_wrap_threshold_counts_unicode_characters() {
        let lines = wrap_coach_text(&("ą".repeat(29) + " rest"));
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].chars().count(), 30);
        assert_eq!(lines[1], "rest");

        let lines = wrap_coach_text(&("a".repeat(14) + "*b"));
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a".repeat(14));
        assert_eq!(lines[1], "b");
    }
}
