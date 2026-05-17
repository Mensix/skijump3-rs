use crate::competition::machine::Competition;
use crate::competition::types::{
    CompetitionPhase, Participant, QualificationStatus, DID_NOT_START_SCORE,
};
use crate::components::screen::{new_screen, page_hints};
use crate::gfx::palette::{FONT_DEFAULT, FONT_GREET, FONT_HEADER};
use crate::store::ResourcesRef;
use engine::ui::Element;

pub const QUALIFICATION_ITEMS_PER_PAGE: usize = 25;

const START_Y: i32 = 23;
const ROW_STEP_QUALIFICATION: i32 = 7;
const COL_RANK: i32 = 24;
const COL_NAME: i32 = 32;
const COL_POINTS: i32 = 184;
const COL_DISTANCE: i32 = 199;
const COL_QUAL: i32 = 252;
const COL_EXTRA: i32 = 275;

const OTHER_NAME: u8 = 241;
const OTHER_RANK: u8 = 251;
const OTHER_DISTANCE: u8 = 252;
const INJURY_COLOR: u8 = 249;

pub struct ResultsPage {
    pub(crate) phase: CompetitionPhase,
    pub(crate) page: usize,
    pub(crate) total_pages: usize,
    pub(crate) items: Vec<ResultsEntry>,
}

pub struct ResultsEntry {
    pub(crate) is_own: bool,
    pub(crate) rank: usize,
    pub(crate) name: String,
    pub(crate) points: i32,
    pub(crate) distance: i32,
    pub(crate) distance2: i32,
    pub(crate) qual: QualificationStatus,
    pub(crate) injury: u8,
}

pub fn build_results_page(competition: &Competition, page: usize) -> ResultsPage {
    let standings = standings_for_phase(competition);
    let total_pages = standings
        .len()
        .div_ceil(QUALIFICATION_ITEMS_PER_PAGE)
        .max(1);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * QUALIFICATION_ITEMS_PER_PAGE;
    let end = (start + QUALIFICATION_ITEMS_PER_PAGE).min(standings.len());

    let mut items = Vec::with_capacity(end - start);
    for &p in &standings[start..end] {
        let (points, dist, dist2) = match competition.phase() {
            CompetitionPhase::QualificationResults => (p.points, p.qual_len, 0),
            CompetitionPhase::Round1Results => (p.points, p.round1_len, 0),
            CompetitionPhase::Round2Results => (p.points, p.round1_len, p.round2_len),
            CompetitionPhase::WorldCupStandings => (p.wc_points, 0, 0),
            _ => (p.points, 0, 0),
        };
        items.push(ResultsEntry {
            is_own: !p.is_computer,
            rank: p.rank,
            name: p.display_name().to_string(),
            points,
            distance: dist,
            distance2: dist2,
            qual: p.qual,
            injury: p.injury,
        });
    }

    ResultsPage {
        phase: competition.phase(),
        page,
        total_pages,
        items,
    }
}

pub fn total_pages(competition: &Competition) -> usize {
    standings_for_phase(competition)
        .len()
        .div_ceil(QUALIFICATION_ITEMS_PER_PAGE)
        .max(1)
}

fn standings_for_phase(competition: &Competition) -> Vec<&Participant> {
    match competition.phase() {
        CompetitionPhase::WorldCupStandings => competition
            .overall_standings()
            .into_iter()
            .filter(|p| p.wc_points > 0)
            .collect(),
        CompetitionPhase::Round1Results => competition
            .event_standings()
            .into_iter()
            .filter(|p| p.points != DID_NOT_START_SCORE)
            .collect(),
        CompetitionPhase::Round2Results => competition
            .event_standings()
            .into_iter()
            .filter(|p| p.qual.can_jump())
            .collect(),
        _ => competition.event_standings(),
    }
}

pub fn render_header(competition: &Competition, resources: &ResourcesRef) -> Vec<Element> {
    let lang = &resources.langbase;
    let event = competition.current_event + 1;
    let total = competition.total_events().max(1);
    let hill_idx = competition.current_hill();
    let hill_str = resources
        .hills
        .hill(hill_idx)
        .map(|h| format!("{} K{}", h.name, h.kr))
        .unwrap_or_default();

    let header = match competition.phase() {
        CompetitionPhase::QualificationResults => {
            format!(
                "{} {} {} {} - {}",
                lang.lstr(82),
                event,
                lang.lstr(8),
                total,
                hill_str
            )
        }
        CompetitionPhase::Round1Results => {
            round_header(lang.lstr(81), event, lang.lstr(8), total, &hill_str, 1)
        }
        CompetitionPhase::Round2Results => {
            round_header(lang.lstr(81), event, lang.lstr(8), total, &hill_str, 2)
        }
        CompetitionPhase::WorldCupStandings => format!(
            "{} {} {} {} {}",
            lang.lstr(27),
            lang.lstr(87),
            event,
            lang.lstr(8),
            total
        ),
        _ => String::new(),
    };

    vec![Element::text(header, 30, 6, FONT_DEFAULT, false)]
}

fn round_header(
    prefix: &str,
    event: usize,
    of: &str,
    total: usize,
    hill: &str,
    round: usize,
) -> String {
    format!("{prefix} {event} {of} {total} - {hill} - R {round}")
}

pub fn render_results_page(page: &ResultsPage, resources: &ResourcesRef) -> Vec<Element> {
    let mut els = new_screen(1);

    els.extend(page_hints(
        page.page,
        page.total_pages,
        resources.langbase.lstr(246),
        resources.langbase.lstr(247),
        resources.langbase.lstr(248),
    ));

    let mut last_rank = 0;
    for (i, entry) in page.items.iter().enumerate() {
        let y = START_Y + i as i32 * ROW_STEP_QUALIFICATION;
        if y > 191 {
            break;
        }

        let (col_text, col_rank, col_dist) = if entry.is_own {
            (FONT_DEFAULT, FONT_HEADER, FONT_GREET)
        } else {
            (OTHER_NAME, OTHER_RANK, OTHER_DISTANCE)
        };

        if entry.rank != last_rank {
            els.push(Element::text(
                format!("{}.", entry.rank),
                COL_RANK,
                y,
                col_rank,
                true,
            ));
        }
        last_rank = entry.rank;

        els.push(Element::text(
            truncate_name(&entry.name),
            COL_NAME,
            y,
            col_text,
            false,
        ));

        let points = if entry.distance == 0 && entry.distance2 == 0 {
            entry.points.to_string()
        } else {
            format_tenths(entry.points)
        };
        els.push(Element::text(points, COL_POINTS, y, col_text, true));

        if entry.distance > 0 {
            els.push(Element::text(
                format_distance(entry.distance, entry.distance2),
                COL_DISTANCE,
                y,
                col_dist,
                false,
            ));
        }

        match page.phase {
            CompetitionPhase::QualificationResults => match entry.qual {
                QualificationStatus::Qualified => {
                    els.push(Element::text("Q", COL_QUAL, y, col_rank, false));
                }
                QualificationStatus::PreQualified => {
                    els.push(Element::text("Q WC", COL_QUAL, y, col_dist, false));
                }
                _ => {}
            },
            CompetitionPhase::Round1Results if entry.rank <= 30 => {
                els.push(Element::text("Q", COL_QUAL, y, col_rank, false));
            }
            _ => {}
        }

        if entry.injury > 0 {
            els.push(Element::text(
                format!("INJ-{}", entry.injury.saturating_sub(1)),
                COL_EXTRA,
                y,
                INJURY_COLOR,
                false,
            ));
        }
    }

    els
}

fn truncate_name(name: &str) -> String {
    const MAX_CHARS: usize = 20;
    if name.chars().count() <= MAX_CHARS {
        return name.to_string();
    }

    name.chars().take(MAX_CHARS).collect()
}

fn format_tenths(value: i32) -> String {
    if value == 0 {
        return "0.0".to_string();
    }

    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    format!("{}{}.{}", sign, abs / 10, abs % 10)
}

fn format_distance(value: i32, value2: i32) -> String {
    let mut length = format_tenths(value);
    while length.len() < 5 {
        length.insert(0, '$');
    }
    if value2 == 0 {
        return format!("({length}µ)");
    }

    let mut second = format_tenths(value2);
    while second.len() < 5 {
        second.insert(0, '$');
    }
    format!("({length}-{second}µ)")
}
