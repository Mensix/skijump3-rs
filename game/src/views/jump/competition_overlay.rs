use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};
use crate::controllers::competition_ui::CompetitionUiState;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::gfx::sprites;
use crate::jump::types::JumpPhase;
use crate::store::ResourcesRef;
use engine::ui::Element;

/// Lightweight snapshot of competition data for overlay rendering.
/// Built once per frame to avoid repeated store.read() calls.
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
    pub points: i32,
}

#[derive(Debug, Clone)]
pub struct WcStandingEntry {
    pub name: String,
    pub points: i32,
}

impl OverlayData {
    /// Collect all data the overlay needs from the competition store.
    pub fn collect(store: &crate::store::StoreRef) -> Option<Self> {
        store.competition.try_with(|c| {
            let event_standings = c.event_standings();
            let event_top5 = event_standings
                .iter()
                .take(5)
                .filter_map(|p| {
                    p.points
                        .filter(|&pts| pts > 0)
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

/// Default key names matching input bindings (K[1..5]).
const KEY_NAMES: [&str; 5] = ["ARROW UP", "ARROW RIGHT", "ARROW LEFT", "T", "R"];

/// Renders overlays (keymap, cycling info, jumper info box) on top of the
/// jump scene during World Cup competition phases. Pure data-in/elements-out.
pub struct CompetitionOverlay {
    resources: ResourcesRef,
    store: crate::store::StoreRef,
}

impl CompetitionOverlay {
    pub fn new(resources: ResourcesRef, store: crate::store::StoreRef) -> Self {
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
            OverlayKind::Keymap => self.drawkeymap_elements(&mut els),
            OverlayKind::CyclingWithInfoBox => {
                self.cycling_info_elements(&mut els, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                self.jumper_info_box(&mut els, &ctx.participant, false);
            }
            OverlayKind::Round2WithInfoBox => {
                self.cycling_info_elements(&mut els, ctx.frame_counter, ctx.hill_idx, &ctx.data);
                self.jumper_info_box(&mut els, &ctx.participant, true);
            }
        }
        els
    }

    /// Pascal drawkeymap: key binding hints shown when jumper is on bar.
    fn drawkeymap_elements(&self, els: &mut Vec<Element>) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        els.push(Element::right_text(
            self.resources.langbase.lstr(330),
            308,
            9,
            FONT_GOLD,
        ));
        for i in 1..=5 {
            els.push(Element::right_text(
                format!(
                    "{}: {}",
                    self.resources.langbase.lstr(330 + i),
                    KEY_NAMES[i - 1]
                ),
                308,
                i as i32 * 10 + 9,
                FONT_GOLD,
            ));
        }
    }

    /// Pascal JumperInfoBox at (3,150).
    fn jumper_info_box(
        &self,
        els: &mut Vec<Element>,
        participant: &Participant,
        round2_with_r1: bool,
    ) {
        els.push(Element::sprite(
            sprites::Sprite::JumperInfoBox as u16,
            3,
            150,
        ));
        let label56 = self.resources.langbase.lstr(56);
        let label56_w = self.resources.font.string_width(label56) as i32;

        let (phase, rank, quali_wc) = self
            .store
            .competition
            .try_with(|c| {
                let phase = c.phase();
                let rank = if round2_with_r1 {
                    participant.round1_rank
                } else {
                    let standings = c.event_standings();
                    standings
                        .iter()
                        .position(|p| p.id == participant.id)
                        .map(|i| i + 1)
                        .unwrap_or(0)
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

        els.push(Element::text(phase_label, 12, 160, FONT_GREET, false));
        els.push(Element::text(label56, 12, 172, FONT_GREET, false));

        let name = if quali_wc {
            format!("{} Q WC", participant.display_name())
        } else if round2_with_r1 && rank > 0 {
            format!("{} ({}.)", participant.display_name(), rank)
        } else {
            participant.display_name().to_string()
        };
        els.push(Element::text(
            name,
            12 + label56_w,
            172,
            FONT_DEFAULT,
            false,
        ));

        if round2_with_r1 {
            let r1text = format!(
                "{} ({}µ)",
                fmt_tenths(participant.round1_score),
                fmt_tenths(participant.round1_len)
            );
            els.push(Element::text(r1text, 14 + label56_w, 179, FONT_HELP, false));
        }
        els.push(Element::text(
            self.resources.langbase.lstr(59),
            12,
            191,
            FONT_HELP,
            false,
        ));
    }

    /// Pascal drawinfo: cycling info on the InfoPanel.
    fn cycling_info_elements(
        &self,
        els: &mut Vec<Element>,
        frame_counter: i32,
        hill_idx: usize,
        data: &OverlayData,
    ) {
        let has_wc_leader = data
            .wc_standings_top5
            .first()
            .map(|e| e.points > 0)
            .unwrap_or(false);
        let has_event_leader = data
            .event_standings_top5
            .first()
            .map(|e| e.points > 0)
            .unwrap_or(false);

        if !has_event_leader {
            if has_wc_leader {
                let phase = (frame_counter as usize) % 292;
                if phase <= 130 {
                    self.hill_info_elements(els, hill_idx);
                } else if (146..=276).contains(&phase) {
                    self.wc_standings_elements(els, &data);
                } else {
                    els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
                }
            } else {
                self.hill_info_elements(els, hill_idx);
            }
            return;
        }

        let cycle = if has_wc_leader { 438 } else { 292 };
        let phase = (frame_counter as usize) % cycle;

        if phase <= 130 {
            self.top5_event_elements(els, &data);
        } else if (146..=276).contains(&phase) {
            self.hill_info_elements(els, hill_idx);
        } else if has_wc_leader && (292..=422).contains(&phase) {
            self.wc_standings_elements(els, &data);
        } else {
            els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        }
    }

    /// Pascal drawtop5info: hill name + top 5 event points with gap behind leader
    fn top5_event_elements(&self, els: &mut Vec<Element>, data: &OverlayData) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
        let hill_name_k = self
            .resources
            .hills
            .hill(data.current_hill)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        els.push(Element::text(hill_name_k, 308, 9, FONT_GOLD, true));

        for (i, entry) in data.event_standings_top5.iter().enumerate() {
            if entry.points > 0 {
                els.push(Element::text(
                    format!("{}  {}", entry.name, fmt_tenths(entry.points)),
                    308,
                    20 + i as i32 * 7,
                    FONT_GOLD,
                    true,
                ));
            }
        }

        // Gap-to-leader line
        if let Some(ref pel) = data.current_participant {
            let leader_pts = data
                .event_standings_top5
                .first()
                .map(|e| e.points)
                .unwrap_or(0);
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
        els.push(Element::text(hill_name_k, 308, 9, FONT_GOLD, true));
        els.push(Element::text(
            self.resources.langbase.lstr(65).to_string(),
            308,
            19,
            FONT_GOLD,
            true,
        ));
        let records = self.store.records.borrow();
        if let Some(r) = records.hill_record(hill_idx) {
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

    /// Pascal drawwcinfo: top 5 WC / season standings with raw points.
    fn wc_standings_elements(&self, els: &mut Vec<Element>, data: &OverlayData) {
        els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
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

fn fmt_tenths(val: i32) -> String {
    let sign = if val < 0 { "-" } else { "" };
    let abs = val.abs();
    format!("{}{}.{}", sign, abs / 10, abs % 10)
}
