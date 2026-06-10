use crate::competition::runtime::CompetitionRuntime;
use crate::competition::team_cup::types::{TeamCupRuntime, TeamCupStandingsKind};
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};
use crate::gfx::palette::{FONT_GOLD, FONT_HELP};
use crate::jump::hud;
use crate::jump::types::JumpPhase;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_decimal;
use crate::views::jump::competition::ui_state::CompetitionUiState;
use engine::ui::Element;

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
    pub fn collect(store: &StoreRef) -> Option<Self> {
        if let Some(data) = Self::collect_wc(store) {
            return Some(data);
        }
        Self::collect_tc(store)
    }

    fn collect_wc(store: &StoreRef) -> Option<Self> {
        store.try_with_competition(|c| {
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

            OverlayData {
                phase: c.phase(),
                style: c.style(),
                current_event: c.current_event,
                current_hill: c.current_hill(),
                current_participant: c.current_jumper().map(|idx| c.participant(idx).clone()),
                event_standings_top5: event_top5,
                wc_standings_top5: wc_top5,
            }
        })
    }

    fn collect_tc(store: &StoreRef) -> Option<Self> {
        store.try_with_team_cup(|tc| {
            let leg_standings = tc.standings_runtime(TeamCupStandingsKind::Leg);
            let overall_standings = tc.standings_runtime(TeamCupStandingsKind::Overall);
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
            let overall_top5: Vec<EventStandingEntry> = overall_standings
                .iter()
                .take(5)
                .filter(|e| e.primary_score > 0.0)
                .map(|e| EventStandingEntry {
                    name: e.name.clone(),
                    points: e.primary_score,
                })
                .collect();
            OverlayData {
                phase: CompetitionPhase::Round1,
                style: CupStyle::TeamCup,
                current_event: tc.current_leg,
                current_hill: hill_idx,
                current_participant: None,
                event_standings_top5: event_top5,
                wc_standings_top5: Vec::new(),
            }
        })
    }
}

/// All data needed to render an overlay on top of the jump scene.
pub struct OverlayContext {
    pub kind: OverlayKind,
    pub participant: Participant,
    pub hill_idx: usize,
    pub frame_counter: i32,
    pub data: OverlayData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    None,
    Keymap,
    CyclingWithInfoBox,
    Round2WithInfoBox,
}

/// Renders overlays (keymap, cycling info, jumper info box) on top of the
/// jump scene during World Cup competition phases. Pure data-in/elements-out.
pub struct CompetitionOverlay {
    resources: ResourcesRef,
    store: StoreRef,
}

impl CompetitionOverlay {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self { resources, store }
    }

    /// Determine what overlay to draw, without rendering.
    pub fn context(
        &self,
        scene_phase: Option<JumpPhase>,
        frame_counter: i32,
        ui_state: &CompetitionUiState,
    ) -> Option<OverlayContext> {
        let scene_phase = scene_phase?;
        let data = OverlayData::collect(&self.store)?;
        let kind = self.resolve_kind(&data, scene_phase, ui_state);
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
        })
    }

    fn resolve_kind(
        &self,
        data: &OverlayData,
        scene_phase: JumpPhase,
        ui_state: &CompetitionUiState,
    ) -> OverlayKind {
        if scene_phase == JumpPhase::Disqualified {
            return OverlayKind::None;
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
    pub fn render_elements(&self, ctx: &OverlayContext) -> Vec<Element> {
        let mut els = Vec::new();
        match ctx.kind {
            OverlayKind::None => {}
            OverlayKind::Keymap => hud::push_keymap(&mut els, &self.resources.langbase),
            OverlayKind::CyclingWithInfoBox => {
                self.cycling_info_elements(&mut els, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(&mut els, &ctx.participant, false);
                }
            }
            OverlayKind::Round2WithInfoBox => {
                self.cycling_info_elements(&mut els, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                if ctx.data.style != CupStyle::TeamCup {
                    self.jumper_info_box(&mut els, &ctx.participant, true);
                }
            }
        }
        els
    }

    /// Pascal `JumperInfoBox` at (3,150).
    fn jumper_info_box(
        &self,
        els: &mut Vec<Element>,
        participant: &Participant,
        round2_with_r1: bool,
    ) {
        let (phase, rank, quali_wc) = self
            .store
            .try_with_competition(|c| {
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
                (phase, rank, quali_wc)
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
            els,
            &self.resources.font,
            &self.resources.langbase,
            &phase_label,
            &name,
            r1text.as_deref().map(|text| (text, FONT_HELP)),
        );
    }

    /// Pascal drawinfo: cycling info on the `InfoPanel`.
    fn cycling_info_elements(
        &self,
        els: &mut Vec<Element>,
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
                    self.hill_info_elements(els, hill_idx);
                } else if (146..=276).contains(&phase) {
                    self.wc_standings_elements(els, data);
                } else {
                    hud::push_info_panel_frame(els);
                }
            } else {
                self.hill_info_elements(els, hill_idx);
            }
            return;
        }

        let cycle = if has_wc_leader { 438 } else { 292 };
        let phase = (frame_counter as usize) % cycle;

        if phase <= 130 {
            self.top5_event_elements(els, data);
        } else if (146..=276).contains(&phase) {
            self.hill_info_elements(els, hill_idx);
        } else if has_wc_leader && (292..=422).contains(&phase) {
            self.wc_standings_elements(els, data);
        } else {
            hud::push_info_panel_frame(els);
        }
    }

    /// Pascal drawtop5info: hill name + top 5 event points with gap behind leader
    fn top5_event_elements(&self, els: &mut Vec<Element>, data: &OverlayData) {
        hud::push_info_panel_frame(els);
        let hill_name_k = self
            .resources
            .hills
            .hill(data.current_hill)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        els.push(Element::text(hill_name_k, 308, 9, FONT_GOLD, true));

        for (i, entry) in data.event_standings_top5.iter().enumerate() {
            if entry.points > 0.0 {
                els.push(Element::text(
                    format!("{}  {}", entry.name, format_decimal(entry.points)),
                    308,
                    20 + i as i32 * 7,
                    FONT_GOLD,
                    true,
                ));
            }
        }

        // Gap-to-leader line
        if let Some(ref pel) = data.current_participant {
            let leader_pts = data.event_standings_top5.first().map_or(0.0, |e| e.points);
            let current_pts = pel.points.unwrap_or(0.0);
            let temp = leader_pts - current_pts;
            if temp > 0.0 {
                let label = self.resources.langbase.lstr(62);
                els.push(Element::text(
                    format!("{}: {}", label, format_decimal(temp + 0.1)),
                    308,
                    62,
                    FONT_GOLD,
                    true,
                ));
            }
        }
    }

    /// Pascal drawhrinfo: hill record name + distance
    fn hill_info_elements(&self, els: &mut Vec<Element>, hill_idx: usize) {
        let hill_name_k = self
            .resources
            .hills
            .hill(hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let records = self.store.records();
        hud::push_hill_record_info(
            els,
            &self.resources.langbase,
            &hill_name_k,
            records.hill_record(hill_idx),
        );
    }

    /// Pascal drawwcinfo: top 5 WC / season standings with raw points.
    fn wc_standings_elements(&self, els: &mut Vec<Element>, data: &OverlayData) {
        hud::push_info_panel_frame(els);
        els.push(Element::text(
            self.resources.langbase.lstr(70).to_string(),
            308,
            9,
            FONT_GOLD,
            true,
        ));
        for (i, entry) in data.wc_standings_top5.iter().enumerate() {
            let s = format!("{}  {}", entry.name, entry.points);
            els.push(Element::text(s, 308, 20 + i as i32 * 7, FONT_GOLD, true));
        }
    }
}
