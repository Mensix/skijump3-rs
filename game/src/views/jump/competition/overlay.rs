use crate::competition::koth::types::KothRuntime;
use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;
use crate::competition::team_cup::types::TeamCupStandingsKind;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};
use crate::competition::ActiveCompetition;
use crate::gfx::sprites::Sprite;
use crate::gfx::theme::{FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::jump::hud;
use crate::jump::types::{JumpPhase, JumpTelemetry};
use crate::store::{GameStateRef, ResourcesRef};
use crate::text::format::format_decimal;
use crate::text::lang::LangBase;
use crate::views::jump::competition::ui_state::CompetitionUiState;
use engine::oxide::PaintCx;

/// Lightweight snapshot of KOTH data for overlay rendering.
#[derive(Debug, Clone)]
pub struct KothOverlayInfo {
    pub alive_count: usize,
    pub total_count: usize,
    pub last_name: String,
    pub last_points: f64,
    pub jump_round: u8,
    pub jump_rounds_per_elimination: u8,
}

/// Lightweight snapshot of competition data for overlay rendering.
/// Built once per frame to avoid repeated `store.read()` calls.
/// Supports both WC/4H (via `Competition`) and Team Cup (via `TeamCupRuntime`).
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
    /// Collect all data the overlay needs from the competition store.
    pub fn collect(store: &GameStateRef) -> Option<Self> {
        let coach_style = active_coach_style(store);
        store
            .borrow()
            .active_competition
            .as_ref()
            .and_then(|active| match active {
                ActiveCompetition::Training => None,
                ActiveCompetition::Individual(comp) => {
                    Some(Self::from_individual(comp, coach_style))
                }
                ActiveCompetition::TeamCup(comp) => Some(Self::from_team_cup(comp, coach_style)),
                ActiveCompetition::Koth(comp) => Some(Self::from_koth(comp, coach_style)),
            })
    }

    fn from_individual(c: &Competition, coach_style: u8) -> Self {
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
            coach_style,
            koth_info: None,
        }
    }

    fn from_koth(c: &KothRuntime, coach_style: u8) -> Self {
        let alive_count = c.participants.iter().filter(|p| p.is_alive()).count();
        let total_count = c.participants.len();
        // Pascal jarjesta5 for KOTH: top5[1] = worst alive (lowest points)
        // Exclude humans (they haven't jumped yet when overlay first appears)
        let last_place = c
            .participants
            .iter()
            .enumerate()
            .filter(|(i, p)| p.is_alive() && !c.human_indices.contains(i))
            .min_by(|(_, a), (_, b)| a.total_points.total_cmp(&b.total_points))
            .map(|(_, p)| p);
        let has_scores = c.participants.iter().any(|p| p.total_points > 0.0);
        let (last_name, last_points) = if has_scores {
            last_place
                .map(|p| (p.competitor.name.clone(), p.total_points))
                .unwrap_or_default()
        } else {
            (String::new(), 0.0)
        };
        Self {
            phase: CompetitionPhase::Round1,
            style: CupStyle::CustomCup,
            current_event: c.current_elimination_round as usize,
            current_hill: c.hill_idx,
            current_participant: None,
            event_standings_top5: Vec::new(),
            wc_standings_top5: Vec::new(),
            coach_style,
            koth_info: Some(KothOverlayInfo {
                alive_count,
                total_count,
                last_name,
                last_points,
                jump_round: c.current_jump_round,
                jump_rounds_per_elimination: c.jump_rounds_per_elimination,
            }),
        }
    }

    fn from_team_cup(tc: &TeamCupRuntime, coach_style: u8) -> Self {
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
            current_participant: None,
            event_standings_top5: event_top5,
            wc_standings_top5: Vec::new(),
            coach_style,
            koth_info: None,
        }
    }
}

fn active_coach_style(store: &GameStateRef) -> u8 {
    let pb = &store.borrow().profiles;
    let idx = match pb.active_order.first() {
        Some(&idx) => idx,
        None => return 0,
    };
    pb.profiles.get(idx).map_or(0, |p| p.coach_style as u8)
}

/// All data needed to render an overlay on top of the jump scene.
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

/// Renders overlays (keymap, cycling info, jumper info box) on top of the
/// jump scene during World Cup competition phases. Pure data-in/elements-out.
pub struct CompetitionOverlay {
    resources: ResourcesRef,
    store: GameStateRef,
}

impl CompetitionOverlay {
    pub fn new(resources: ResourcesRef, store: GameStateRef) -> Self {
        Self { resources, store }
    }

    /// Determine what overlay to draw, without rendering.
    pub fn context(
        &self,
        scene_phase: Option<JumpPhase>,
        frame_counter: i32,
        ui_state: &CompetitionUiState,
        telemetry: Option<JumpTelemetry>,
    ) -> Option<OverlayContext> {
        let scene_phase = scene_phase?;
        let data = OverlayData::collect(&self.store)?;
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
        // Coach corner: show after a human jump result during Info/OnBar
        if let Some(t) = telemetry {
            if t.grade > 0
                && data.coach_style > 0
                && data
                    .current_participant
                    .as_ref()
                    .is_some_and(|p| !p.is_computer)
                && matches!(
                    scene_phase,
                    JumpPhase::Info | JumpPhase::OnBar | JumpPhase::Result
                )
            {
                return OverlayKind::Coach;
            }
        }

        // KOTH overlay: cycle between jumpers left and hill record
        if data.koth_info.is_some()
            && matches!(
                scene_phase,
                JumpPhase::Info | JumpPhase::OnBar | JumpPhase::Result
            )
        {
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
            (CompetitionPhase::Qualification, JumpPhase::Info) if show_keymap => {
                OverlayKind::Keymap
            }
            // Explicitly list phases that get cycling info — not needs_event_results()
            // which would also match Round2 (and CustomCup Round2 must show no overlay).
            (CompetitionPhase::Qualification | CompetitionPhase::Round1, JumpPhase::Info) => {
                OverlayKind::CyclingWithInfoBox
            }
            _ => OverlayKind::None,
        }
    }

    /// Render all overlay elements for the current state.
    pub fn render(&self, cx: &mut PaintCx<'_>, ctx: &OverlayContext) {
        match ctx.kind {
            OverlayKind::None => {}
            OverlayKind::Keymap => hud::push_keymap(cx, &self.resources.langbase),
            OverlayKind::CyclingWithInfoBox => {
                self.cycling_info_elements(cx, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(cx, &ctx.participant, false);
                }
            }
            OverlayKind::Round2WithInfoBox => {
                self.cycling_info_elements(cx, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(cx, &ctx.participant, true);
                }
            }
            OverlayKind::Coach => self.coach_elements(cx, ctx),
            OverlayKind::Koth => {
                if matches!(ctx.scene_phase, JumpPhase::Info | JumpPhase::Result) {
                    let phase = (ctx.frame_counter as usize) % 292;
                    if phase <= 130 {
                        self.koth_info_elements(cx, ctx);
                    } else if (146..=276).contains(&phase) {
                        self.hill_info_elements(cx, ctx.hill_idx);
                    }
                }
            }
        }
    }

    /// Pascal `DoCoachCorner`: coach advice panel at bottom-left after jump.
    fn coach_elements(&self, cx: &mut PaintCx<'_>, ctx: &OverlayContext) {
        let Some(ref t) = ctx.telemetry else { return };
        let style = ctx.data.coach_style;
        if style == 0 || t.grade == 0 {
            return;
        }
        let base = 360 + style as usize * 40;
        let lang = &self.resources.langbase;

        cx.sprite(Sprite::JumperInfoBox as u16, (3, 150));
        cx.text((12, 150), FONT_TEAL, lang.lstr(400));
        cx.text((12, 160), FONT_TEAL, "\"");

        let cstr0 = self.coach_range(lang, base + 2, t.angle_counter, &[49, 61, 200]);
        let cstr1 = if t.grade < 10 {
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
        let mut cstr3 = self.coach_range(lang, base + 28, t.height, &[49, 55, 60, 64, 70, 90, 200]);

        // Pascal: if (grade=1) then cstr[3]:=lstr(index+35);
        if t.grade == 1 {
            cstr3 = lang.lstr(base + 35).to_string();
        }

        let cstr0 = if t.grade < 10 { cstr1.clone() } else { cstr0 };

        let r = (t.grade as u16).wrapping_mul(7)
            ^ (t.height as u16).wrapping_mul(13)
            ^ (t.takeoff_timing as u16).wrapping_mul(31)
            ^ (t.angle_counter as u16).wrapping_mul(61);
        let pick_a = if r & 1 == 0 { &cstr0 } else { &cstr1 };
        let pick_b = if r & 2 == 0 { &cstr2 } else { &cstr3 };

        // Pascal: joined with '*' which acts as space + optional line-break hint
        let text = format!("{pick_a}*{pick_b}");

        // Pascal word-wrap (count=30, half=15):
        //   wstr accumulates chars, * → space in buffer,
        //   break at space when past 30 chars, or at * when past 15 chars.
        let mut y = 152i32;
        let mut line = String::with_capacity(32);
        for ch in text.chars() {
            line.push(if ch == '*' { ' ' } else { ch });
            // Pascal: index = line.len() + 1, check index > count → line.len() >= 30
            if (line.len() >= 30 && ch == ' ') || (ch == '*' && line.len() >= 15) {
                if ch == '*' {
                    line.pop();
                }
                if y < 190 {
                    y += 8;
                }
                cx.text((18, y), FONT_TEAL, line.clone());
                line.clear();
            }
        }
        if !line.is_empty() {
            if line.len() < 2 {
                cx.text((18, y), FONT_TEAL, format!("{line}\""));
            } else {
                if y < 192 {
                    y += 8;
                }
                cx.text((18, y), FONT_TEAL, format!("{line}\""));
            }
        }
    }

    /// Look up language string for a value within the given range thresholds.
    fn coach_range(&self, lang: &LangBase, base: usize, val: u8, thresholds: &[u8]) -> String {
        let idx = thresholds
            .iter()
            .position(|&t| val <= t)
            .unwrap_or(thresholds.len());
        lang.lstr(base + idx).to_string()
    }

    /// Pascal `JumperInfoBox` at (3,150).
    fn jumper_info_box(
        &self,
        cx: &mut PaintCx<'_>,
        participant: &Participant,
        round2_with_r1: bool,
    ) {
        let (phase, rank, quali_wc) = self
            .store
            .borrow()
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
            CompetitionPhase::Training(n) => format!("{} {}", self.resources.langbase.lstr(52), n),
            CompetitionPhase::Qualification => self.resources.langbase.lstr(53).to_string(),
            CompetitionPhase::Round1 => self.resources.langbase.lstr(54).to_string(),
            CompetitionPhase::Round2 => self.resources.langbase.lstr(55).to_string(),
            _ => self.resources.langbase.lstr(51).to_string(),
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

    /// Pascal drawinfo: cycling info on the `InfoPanel`.
    fn cycling_info_elements(
        &self,
        cx: &mut PaintCx<'_>,
        frame_counter: i32,
        hill_idx: usize,
        data: &OverlayData,
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
                    self.hill_info_elements(cx, hill_idx);
                } else if (146..=276).contains(&phase) {
                    self.wc_standings_elements(cx, data);
                } else {
                    hud::push_info_panel_frame(cx);
                }
            } else {
                self.hill_info_elements(cx, hill_idx);
            }
            return;
        }

        let cycle = if has_wc_leader { 438 } else { 292 };
        let phase = (frame_counter as usize) % cycle;

        if phase <= 130 {
            self.top5_event_elements(cx, data);
        } else if (146..=276).contains(&phase) {
            self.hill_info_elements(cx, hill_idx);
        } else if has_wc_leader && (292..=422).contains(&phase) {
            self.wc_standings_elements(cx, data);
        } else {
            hud::push_info_panel_frame(cx);
        }
    }

    /// Pascal drawtop5info: hill name + top 5 event points with gap behind leader
    fn top5_event_elements(&self, cx: &mut PaintCx<'_>, data: &OverlayData) {
        hud::push_info_panel_frame(cx);
        let hill_name_k = self
            .resources
            .hills
            .hill(data.current_hill)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        cx.right_text((308, 9), FONT_GOLD, hill_name_k);

        for (i, entry) in data.event_standings_top5.iter().enumerate() {
            if entry.points > 0.0 {
                cx.right_text(
                    (308, 20 + i as i32 * 7),
                    FONT_GOLD,
                    format!("{}  {}", entry.name, format_decimal(entry.points)),
                );
            }
        }

        // Gap-to-leader line
        if let Some(ref pel) = data.current_participant {
            let leader_pts = data.event_standings_top5.first().map_or(0.0, |e| e.points);
            let current_pts = pel.points.unwrap_or(0.0);
            let temp = leader_pts - current_pts;
            if temp > 0.0 {
                let label = self.resources.langbase.lstr(62);
                cx.right_text(
                    (308, 62),
                    FONT_GOLD,
                    format!("{}: {}", label, format_decimal(temp + 0.1)),
                );
            }
        }
    }

    /// Pascal drawhrinfo: hill record name + distance
    fn hill_info_elements(&self, cx: &mut PaintCx<'_>, hill_idx: usize) {
        let hill_name_k = self
            .resources
            .hills
            .hill(hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let records = &self.store.borrow().records;
        hud::push_hill_record_info(
            cx,
            &self.resources.langbase,
            &hill_name_k,
            records.hill_record(hill_idx),
        );
    }

    /// Pascal drawkothinfo: jumpers left + worst alive with phase label
    fn koth_info_elements(&self, cx: &mut PaintCx<'_>, ctx: &OverlayContext) {
        let lang = &self.resources.langbase;
        let Some(ref ki) = ctx.data.koth_info else {
            return;
        };
        let total = ki.total_count;
        let left = ki.alive_count;
        hud::push_info_panel_frame(cx);
        // "Jumpers Left: N of TOTAL" — Pascal lstr(67) + lstr(8)
        let str1 = format!("{} {} {}", lang.lstr(67), left, lang.lstr(8));
        cx.right_text((308, 9), FONT_GOLD, format!("{str1} {total}"));

        // Phase label + worst-alive info (Pascal top5[1]=lowest points for KOTH)
        if !ki.last_name.is_empty() {
            let label = if ki.jump_round == 0 && ki.jump_rounds_per_elimination > 1 {
                lang.lstr(69) // "Currently Last:"
            } else {
                lang.lstr(68) // "Need to Beat:"
            };
            cx.right_text((308, 19), FONT_GOLD, label);
            let pts_str = format_decimal(ki.last_points);
            cx.right_text(
                (308, 29),
                FONT_GOLD,
                format!("{} ${}", ki.last_name, pts_str),
            );
        }
    }

    /// Pascal drawwcinfo: top 5 WC / season standings with raw points.
    fn wc_standings_elements(&self, cx: &mut PaintCx<'_>, data: &OverlayData) {
        hud::push_info_panel_frame(cx);
        cx.right_text(
            (308, 9),
            FONT_GOLD,
            self.resources.langbase.lstr(70).to_string(),
        );
        for (i, entry) in data.wc_standings_top5.iter().enumerate() {
            let s = format!("{}  {}", entry.name, entry.points);
            cx.right_text((308, 20 + i as i32 * 7), FONT_GOLD, s);
        }
    }
}
