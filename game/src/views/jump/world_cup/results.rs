use crate::competition::machine::Competition;
use crate::competition::types::{
    CompetitionPhase, CupStyle, EventHistoryReason, EventJumpHistory, IndividualEventHistory,
    Participant, QualificationStatus,
};
use crate::components::page_nav::{differential_score, render_page_hints, PageHintLayout};

use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_DARKEST, BG_PURPLE, BG_RED_BRIGHT, BG_WORLDCUP, BLACK, FILL_GOLD, FILL_GRAY, FILL_TEAL,
    FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::store::ResourcesRef;
use crate::text::format::{format_decimal, ordinal_dot};
use crate::text::layout::shorten_name;
use crate::ui::{Font, UiCanvas};
use engine::color::Rgba;

pub const QUALIFICATION_ITEMS_PER_PAGE: usize = 25;
const STATS_EVENTS_PER_PAGE: usize = 8;

fn list_background(phase: CompetitionPhase, style: CupStyle) -> Rgba {
    match phase {
        CompetitionPhase::FourHillsStandings => BG_DARKEST,
        CompetitionPhase::WorldCupStandings => BG_WORLDCUP,
        CompetitionPhase::SeasonComplete => {
            if matches!(style, CupStyle::FourHills | CupStyle::CustomCup) {
                BG_DARKEST
            } else {
                BG_WORLDCUP
            }
        }
        _ => BG_PURPLE,
    }
}

fn competition_list_background(competition: &Competition) -> Rgba {
    if competition.phase() == CompetitionPhase::SeasonComplete
        && competition.style() == CupStyle::CustomCup
        && !competition.uses_aggregate_standings()
    {
        BG_WORLDCUP
    } else {
        list_background(competition.phase(), competition.style())
    }
}

mod ko;
pub use ko::render_ko_pairs;

const WC_ITEMS_PER_PAGE: usize = 44;
const WC_COL_SPLIT: usize = 22;
const WC_ROW_STEP: i32 = 8;

const START_Y: i32 = 23;
const ROW_STEP_QUALIFICATION: i32 = 7;
const COL_RANK: i32 = 24;
const COL_NAME: i32 = 32;
const COL_POINTS: i32 = 184;
const COL_DISTANCE: i32 = 199;
const COL_QUAL: i32 = 252;
const COL_EXTRA: i32 = 275;

const WC_RANK: i32 = 19;
const WC_NAME: i32 = 23;
const WC_POINTS: i32 = 153;
const WC_COL2_OFFSET: i32 = 160;

const OTHER_NAME: Rgba = FONT_GRAY;
const OTHER_RANK: Rgba = FILL_GOLD;
const OTHER_DISTANCE: Rgba = FILL_TEAL;
const INJURY_COLOR: Rgba = BG_RED_BRIGHT;

pub struct ResultsPage {
    pub(crate) phase: CompetitionPhase,
    pub(crate) style: CupStyle,
    pub(crate) page: usize,
    pub(crate) total_pages: usize,
    pub(crate) items: Vec<ResultsEntry>,
    pub(crate) compact: bool,
    pub(crate) prev_last_rank: usize,
    aggregate_standings: bool,
    leader_points: f64,
}

pub struct ResultsEntry {
    pub(crate) is_own: bool,
    pub(crate) rank: usize,
    pub(crate) name: String,
    pub(crate) points: f64,
    pub(crate) distance: f64,
    pub(crate) distance2: f64,
    pub(crate) qual: QualificationStatus,
    pub(crate) injury: u8,
    pub(crate) use_tenths: bool,
}

fn use_decimal_for_phase(competition: &Competition) -> bool {
    let phase = competition.phase();
    matches!(
        phase,
        CompetitionPhase::QualificationResults
            | CompetitionPhase::Round1Results
            | CompetitionPhase::Round2Results
            | CompetitionPhase::FourHillsStandings
    ) || phase == CompetitionPhase::SeasonComplete && competition.uses_aggregate_standings()
}

fn competition_results_entry_data(competition: &Competition, p: &Participant) -> (f64, f64, f64) {
    let phase = competition.phase();
    match phase {
        CompetitionPhase::QualificationResults => (p.points.unwrap_or(0.0), p.qual_len, 0.0),
        CompetitionPhase::Round1Results => (p.points.unwrap_or(0.0), p.round1_len, 0.0),
        CompetitionPhase::Round2Results => (p.points.unwrap_or(0.0), p.round1_len, p.round2_len),
        CompetitionPhase::FourHillsStandings => (p.four_hills_points, 0.0, 0.0),
        CompetitionPhase::WorldCupStandings => (f64::from(p.wc_points), 0.0, 0.0),
        CompetitionPhase::SeasonComplete if competition.uses_aggregate_standings() => {
            (p.four_hills_points, 0.0, 0.0)
        }
        CompetitionPhase::SeasonComplete => (f64::from(p.wc_points), 0.0, 0.0),
        _ => (p.points.unwrap_or(0.0), 0.0, 0.0),
    }
}

fn items_per_page(phase: CompetitionPhase) -> usize {
    if matches!(
        phase,
        CompetitionPhase::FourHillsStandings
            | CompetitionPhase::WorldCupStandings
            | CompetitionPhase::SeasonComplete
    ) {
        WC_ITEMS_PER_PAGE
    } else if phase.result_round_number().is_some() {
        22
    } else {
        QUALIFICATION_ITEMS_PER_PAGE
    }
}

pub fn build_results_page(competition: &Competition, page: usize) -> ResultsPage {
    let standings = standings_for_phase(competition);
    let per_page = items_per_page(competition.phase());
    let ranges = result_page_ranges(&standings, competition.phase(), per_page);
    let total_pages = ranges.len().max(1);
    let page = page.min(total_pages - 1);
    let (start, end) = ranges.get(page).copied().unwrap_or((0, 0));

    let use_tenths = use_decimal_for_phase(competition);
    let mut items = Vec::with_capacity(end - start);
    for &p in &standings[start..end] {
        let (points, dist, dist2) = competition_results_entry_data(competition, p);
        items.push(ResultsEntry {
            is_own: !p.is_computer,
            rank: p.rank,
            name: p.display_name().to_string(),
            points,
            distance: dist,
            distance2: dist2,
            qual: p.qual,
            injury: p.injury,
            use_tenths,
        });
    }

    let prev_last_rank = if page > 0 {
        standings.get(start - 1).map_or(0, |p| p.rank)
    } else {
        0
    };
    ResultsPage {
        phase: competition.phase(),
        style: competition.style(),
        page,
        total_pages,
        items,
        compact: false,
        prev_last_rank,
        aggregate_standings: competition.uses_aggregate_standings(),
        leader_points: standings
            .first()
            .map_or(0.0, |p| competition_results_entry_data(competition, p).0),
    }
}

pub fn build_compact_results_page(competition: &Competition) -> ResultsPage {
    let standings = standings_for_phase(competition);
    let mut selected: Vec<&Participant> = standings.iter().take(10).copied().collect();
    for &p in &standings {
        if !p.is_computer && !selected.iter().any(|existing| existing.id == p.id) {
            selected.push(p);
        }
    }

    let use_tenths = use_decimal_for_phase(competition);
    let mut items = Vec::with_capacity(selected.len());
    for p in selected {
        let (points, dist, dist2) = competition_results_entry_data(competition, p);
        items.push(ResultsEntry {
            is_own: !p.is_computer,
            rank: p.rank,
            name: p.display_name().to_string(),
            points,
            distance: dist,
            distance2: dist2,
            qual: p.qual,
            injury: p.injury,
            use_tenths,
        });
    }

    ResultsPage {
        phase: competition.phase(),
        style: competition.style(),
        page: 0,
        total_pages: 1,
        items,
        compact: true,
        prev_last_rank: 0,
        aggregate_standings: competition.uses_aggregate_standings(),
        leader_points: standings
            .first()
            .map_or(0.0, |p| competition_results_entry_data(competition, p).0),
    }
}

pub fn total_pages(competition: &Competition) -> usize {
    let per_page = items_per_page(competition.phase());
    result_page_ranges(
        &standings_for_phase(competition),
        competition.phase(),
        per_page,
    )
    .len()
    .max(1)
}

fn result_page_ranges(
    standings: &[&Participant],
    phase: CompetitionPhase,
    per_page: usize,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < standings.len() {
        let mut end = start;
        let mut slots = 0;
        while end < standings.len() && slots < per_page {
            let separator = phase == CompetitionPhase::Round1Results
                && standings[end].rank > 30
                && ((end == start && start > 0 && standings[start - 1].rank <= 30)
                    || (end > start && standings[end - 1].rank <= 30));
            slots += 1 + usize::from(separator);
            if slots > per_page {
                break;
            }
            end += 1;
        }
        ranges.push((start, end));
        start = end;
    }
    ranges
}

fn standings_for_phase(competition: &Competition) -> Vec<&Participant> {
    let phase = competition.phase();
    if phase == CompetitionPhase::FourHillsStandings
        || phase == CompetitionPhase::SeasonComplete && competition.uses_aggregate_standings()
    {
        competition
            .overall_standings()
            .into_iter()
            .filter(|p| p.four_hills_points > 0.0)
            .collect()
    } else if matches!(
        phase,
        CompetitionPhase::WorldCupStandings | CompetitionPhase::SeasonComplete
    ) {
        competition
            .overall_standings()
            .into_iter()
            .filter(|p| p.wc_points > 0)
            .collect()
    } else if phase == CompetitionPhase::Round2Results {
        competition
            .event_standings()
            .into_iter()
            .filter(|p| p.qual.can_jump())
            .collect()
    } else if phase == CompetitionPhase::QualificationResults
        || phase == CompetitionPhase::Round1Results
    {
        competition
            .event_standings()
            .into_iter()
            .filter(|p| {
                p.points.is_some()
                    || (phase == CompetitionPhase::QualificationResults
                        && matches!(
                            p.qual,
                            QualificationStatus::PreQualified | QualificationStatus::KoSeed(_)
                        ))
            })
            .collect()
    } else {
        competition.event_standings()
    }
}

pub fn render_header(cx: &mut dyn UiCanvas, competition: &Competition, resources: &ResourcesRef) {
    let lang = &resources.langbase;
    let event = competition.current_event + 1;
    let total = competition.total_events().max(1);
    let hill_idx = competition.current_hill();
    let hill_str = resources
        .hills
        .hill(hill_idx)
        .map(|h| format!("{} K{}", h.name, h.kr))
        .unwrap_or_default();

    let cup_style_str = |style: CupStyle, offset: usize| -> String {
        let cup_idx = match style {
            CupStyle::WorldCup => 0,
            CupStyle::CustomCup => 1,
            CupStyle::FourHills => 2,
            CupStyle::TeamCup => 0,
        };
        lang.tr(offset + cup_idx).to_string()
    };

    let phase = competition.phase();
    let header = match phase {
        _ if phase == CompetitionPhase::QualificationResults => {
            format!(
                "{} {} {} {} - {}",
                lang.tr(82),
                event,
                lang.tr(8),
                total,
                hill_str
            )
        }
        _ if phase == CompetitionPhase::Round1Results => {
            round_header(lang.tr(81), event, lang.tr(8), total, &hill_str, 1)
        }
        _ if phase == CompetitionPhase::Round2Results => {
            round_header(lang.tr(81), event, lang.tr(8), total, &hill_str, 2)
        }
        _ if phase == CompetitionPhase::FourHillsStandings => {
            if competition.style() == CupStyle::FourHills && event >= total {
                lang.tr(85).to_string()
            } else if competition.style() == CupStyle::FourHills {
                format!(
                    "{} {} {} {} - {}",
                    lang.tr(84),
                    event,
                    lang.tr(8),
                    total,
                    hill_str
                )
            } else {
                let prefix = cup_style_str(competition.style(), 27);
                format!(
                    "{} {} {} {} {}",
                    prefix,
                    lang.tr(87),
                    event,
                    lang.tr(8),
                    total
                )
            }
        }
        _ if phase == CompetitionPhase::WorldCupStandings => {
            let prefix = cup_style_str(competition.style(), 27);
            format!(
                "{} {} {} {} {}",
                prefix,
                lang.tr(87),
                event,
                lang.tr(8),
                total
            )
        }
        _ if phase == CompetitionPhase::SeasonComplete => {
            if competition.style() == CupStyle::FourHills {
                lang.tr(85).to_string()
            } else {
                format!("{} {}", lang.tr(90), cup_style_str(competition.style(), 27))
            }
        }
        _ => String::new(),
    };

    cx.text((30, 6), FONT_BODY, &header);
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

struct EntryRenderCx<'a> {
    cx: &'a mut dyn UiCanvas,
    font: &'a Font,
    last_rank: &'a mut usize,
}

struct EntryColumns {
    y: i32,
    rank_x: i32,
    name_x: i32,
    points_x: i32,
}

fn render_results_entry(
    rcx: EntryRenderCx<'_>,
    entry: &ResultsEntry,
    phase: CompetitionPhase,
    cols: EntryColumns,
    show_extra: bool,
    points: f64,
) {
    let EntryRenderCx {
        cx,
        font,
        last_rank,
    } = rcx;
    let EntryColumns {
        y,
        rank_x,
        name_x,
        points_x,
    } = cols;
    let (col_text, col_rank, col_dist) = if entry.is_own {
        (FONT_BODY, FONT_GOLD, FONT_TEAL)
    } else {
        (OTHER_NAME, OTHER_RANK, OTHER_DISTANCE)
    };

    if entry.rank != *last_rank {
        cx.right_text((rank_x, y), col_rank, &ordinal_dot(entry.rank));
    }
    *last_rank = entry.rank;

    cx.text(
        (name_x, y),
        col_text,
        &shorten_name(&entry.name, font, points_x - name_x - 5),
    );

    if entry.use_tenths {
        cx.right_text((points_x, y), col_text, &format_decimal(points));
    } else {
        cx.right_text((points_x, y), col_text, &format!("{points:.0}"));
    }

    if show_extra && entry.distance > 0.0 {
        cx.text(
            (COL_DISTANCE, y),
            col_dist,
            &format_distance(entry.distance, entry.distance2),
        );
    }

    if show_extra {
        if phase == CompetitionPhase::QualificationResults {
            match entry.qual {
                QualificationStatus::Qualified => {
                    cx.text((COL_QUAL, y), col_rank, "Q");
                }
                QualificationStatus::PreQualified => {
                    cx.text((COL_QUAL, y), col_dist, "Q WC");
                }
                QualificationStatus::KoSeed(_) => {
                    cx.text((COL_QUAL, y), col_rank, "Q");
                }
                _ => {}
            }
        } else if phase == CompetitionPhase::Round1Results && entry.rank <= 30 {
            cx.text((COL_QUAL, y), col_rank, "Q");
        }

        if entry.injury > 0 {
            cx.text(
                (COL_EXTRA, y),
                INJURY_COLOR,
                &format!("INJ-{}", entry.injury.saturating_sub(1)),
            );
        }
    }
}

pub fn render_results_page(
    cx: &mut dyn UiCanvas,
    page: &ResultsPage,
    resources: &ResourcesRef,
    event_gap: bool,
    wc_gap: bool,
    hill_background: bool,
) {
    let lang = &resources.langbase;
    let bg = if page.phase == CompetitionPhase::SeasonComplete
        && page.style == CupStyle::CustomCup
        && !page.aggregate_standings
    {
        BG_WORLDCUP
    } else {
        list_background(page.phase, page.style)
    };
    if !hill_background {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 20, 320, 180), bg);
    }
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    render_page_hints(cx, page.page, page.total_pages, lang, PageHintLayout::Top);

    if page.total_pages == 1 && page.items.len() <= 20 {
        cx.text((30, 190), FONT_TEAL, lang.tr(86));
    }

    let is_wc = page.phase == CompetitionPhase::FourHillsStandings
        || page.phase == CompetitionPhase::WorldCupStandings
        || page.phase == CompetitionPhase::SeasonComplete;
    let differential = if is_wc { wc_gap } else { event_gap };

    let row_step = if is_wc {
        WC_ROW_STEP
    } else if page.phase.result_round_number().is_some() {
        8
    } else {
        ROW_STEP_QUALIFICATION
    };

    let mut last_rank = 0;
    if is_wc {
        for (i, entry) in page.items.iter().enumerate() {
            let col = i32::from(i >= WC_COL_SPLIT);
            let col_off = col * WC_COL2_OFFSET;
            let y = START_Y + (i % WC_COL_SPLIT) as i32 * row_step;
            render_results_entry(
                EntryRenderCx {
                    cx: &mut *cx,
                    font: &resources.font,
                    last_rank: &mut last_rank,
                },
                entry,
                page.phase,
                EntryColumns {
                    y,
                    rank_x: WC_RANK + col_off,
                    name_x: WC_NAME + col_off,
                    points_x: WC_POINTS + col_off,
                },
                false,
                differential_score(page.leader_points, entry.points, entry.rank, differential),
            );
        }
    } else {
        let mut y = START_Y;
        for (i, entry) in page.items.iter().enumerate() {
            if y > 191 {
                break;
            }

            if !page.compact
                && page.phase == CompetitionPhase::Round1Results
                && entry.rank > 30
                && (i > 0 && page.items[i - 1].rank <= 30 || i == 0 && page.prev_last_rank <= 30)
            {
                let half = row_step / 2;
                cx.text((COL_NAME, y + half), FONT_GOLD, "- - -");
                y += half + row_step + half;
                if y > 191 {
                    break;
                }
            }

            render_results_entry(
                EntryRenderCx {
                    cx: &mut *cx,
                    font: &resources.font,
                    last_rank: &mut last_rank,
                },
                entry,
                page.phase,
                EntryColumns {
                    y,
                    rank_x: COL_RANK,
                    name_x: COL_NAME,
                    points_x: COL_POINTS,
                },
                true,
                differential_score(page.leader_points, entry.points, entry.rank, differential),
            );
            y += row_step;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsTotals {
    pub events: usize,
    pub event_points: f64,
    pub wc_points: i32,
    pub average_event_points: f64,
    pub qualification_points: f64,
    pub round1_points: f64,
    pub round2_points: f64,
    pub average_qualification_points: Option<f64>,
    pub average_round1_points: Option<f64>,
    pub average_round2_points: Option<f64>,
    pub average_qualification_distance: Option<f64>,
    pub average_round1_distance: Option<f64>,
    pub average_round2_distance: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsPage {
    pub player_name: String,
    pub page: usize,
    pub total_pages: usize,
    pub events: Vec<IndividualEventHistory>,
    pub totals: StatsTotals,
}

fn human_stats_pages(competition: &Competition) -> Vec<(&Participant, usize)> {
    competition
        .overall_standings()
        .into_iter()
        .filter(|p| !p.is_computer)
        .map(|p| {
            (
                p,
                p.event_history.len().div_ceil(STATS_EVENTS_PER_PAGE).max(1),
            )
        })
        .collect()
}

pub fn stats_total_pages(competition: &Competition) -> usize {
    human_stats_pages(competition)
        .iter()
        .map(|(_, pages)| pages)
        .sum::<usize>()
}

pub fn build_stats_page(competition: &Competition, requested_page: usize) -> Option<StatsPage> {
    let players = human_stats_pages(competition);
    let total_pages = players.iter().map(|(_, pages)| pages).sum::<usize>().max(1);
    let page = requested_page.min(total_pages - 1);
    let mut first_page = 0;
    let (player, player_page) = players.into_iter().find_map(|(player, pages)| {
        let selected = (page < first_page + pages).then_some((player, page - first_page));
        first_page += pages;
        selected
    })?;
    let start = player_page * STATS_EVENTS_PER_PAGE;
    let events = player.event_history
        [start..(start + STATS_EVENTS_PER_PAGE).min(player.event_history.len())]
        .to_vec();
    let event_points: f64 = player
        .event_history
        .iter()
        .filter_map(|event| event.event_points)
        .sum();
    let average = |values: Vec<f64>| {
        (!player.event_history.is_empty())
            .then(|| values.iter().sum::<f64>() / player.event_history.len() as f64)
    };
    let qualification_distances: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.qualification.map(|jump| jump.distance))
        .collect();
    let qualification_points: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.qualification.map(|jump| jump.points))
        .collect();
    let round1_points: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.round1.map(|jump| jump.points))
        .collect();
    let round2_points: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.round2.map(|jump| jump.points))
        .collect();
    let round1_distances: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.round1.map(|jump| jump.distance))
        .collect();
    let round2_distances: Vec<_> = player
        .event_history
        .iter()
        .filter_map(|event| event.round2.map(|jump| jump.distance))
        .collect();
    Some(StatsPage {
        player_name: player.display_name().to_string(),
        page,
        total_pages,
        events,
        totals: StatsTotals {
            events: player.event_history.len(),
            event_points,
            wc_points: player
                .event_history
                .iter()
                .map(|event| event.wc_points_awarded)
                .sum(),
            average_event_points: if player.event_history.is_empty() {
                0.0
            } else {
                event_points / player.event_history.len() as f64
            },
            qualification_points: qualification_points.iter().sum(),
            round1_points: round1_points.iter().sum(),
            round2_points: round2_points.iter().sum(),
            average_qualification_points: average(qualification_points),
            average_round1_points: average(round1_points),
            average_round2_points: average(round2_points),
            average_qualification_distance: average(qualification_distances),
            average_round1_distance: average(round1_distances),
            average_round2_distance: average(round2_distances),
        },
    })
}

pub fn render_stats_page(
    cx: &mut dyn UiCanvas,
    competition: &Competition,
    resources: &ResourcesRef,
    page: usize,
    hill_background: bool,
) {
    let lang = &resources.langbase;

    let bg = competition_list_background(competition);
    if !hill_background {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 20, 320, 180), bg);
    }
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let Some(stats) = build_stats_page(competition, page) else {
        cx.text((30, 6), FONT_BODY, lang.tr(89));
        return;
    };

    cx.text((30, 6), FONT_BODY, lang.tr(89));
    cx.text(
        (36 + lang.tr(89).len() as i32 * 6, 6),
        FONT_BODY,
        &stats.player_name,
    );
    cx.text((16, 23), FONT_TEAL, "EV HILL");
    cx.right_text((91, 23), FONT_TEAL, lang.tr(108));
    cx.right_text((113, 23), FONT_TEAL, lang.tr(109));
    cx.right_text((139, 23), FONT_TEAL, lang.tr(98));
    cx.right_text((168, 23), FONT_TEAL, lang.tr(97));
    cx.right_text((215, 23), FONT_TEAL, "QUAL");
    cx.right_text((263, 23), FONT_TEAL, "R 1");
    cx.right_text((311, 23), FONT_TEAL, "R 2");

    for (row, event) in stats.events.iter().enumerate() {
        let y = 35 + row as i32 * 18;
        let hill = resources
            .hills
            .hill(event.hill_idx)
            .map(|hill| {
                format!(
                    "{}{}",
                    hill.name.chars().take(3).collect::<String>(),
                    hill.kr
                )
            })
            .unwrap_or_default();
        cx.text(
            (16, y),
            FONT_BODY,
            &format!("{}. {hill}", event.event_index + 1),
        );
        if let Some(placing) = event.placing {
            cx.right_text((91, y), FONT_GOLD, &ordinal_dot(placing));
        }
        cx.right_text((113, y), FONT_BODY, &event.wc_points_awarded.to_string());
        cx.right_text((139, y), FONT_GOLD, &ordinal_dot(event.running_rank));
        if let Some(points) = event.event_points {
            cx.right_text((168, y), FONT_BODY, &format_decimal(points));
        }
        render_history_jump(cx, event.qualification.as_ref(), 215, y);
        render_history_jump(cx, event.round1.as_ref(), 263, y);
        render_history_jump(cx, event.round2.as_ref(), 311, y);

        let mut status = qualification_status(event.qualification_status).to_string();
        let final_status = qualification_status(event.status);
        if final_status != status {
            status.push('/');
            status.push_str(final_status);
        }
        if let Some(EventHistoryReason::Injury(rounds)) = event.reason {
            status.push_str(&format!(" INJ-{}", rounds.saturating_sub(1)));
        }
        cx.text((16, y + 8), FONT_TEAL, &status);
    }

    cx.text(
        (16, 181),
        FONT_GOLD,
        &format!("{} {} EV", lang.tr(100), stats.totals.events),
    );
    cx.text(
        (83, 181),
        FONT_BODY,
        &format!("WC {}", stats.totals.wc_points),
    );
    cx.right_text(
        (168, 181),
        FONT_BODY,
        &format_decimal(stats.totals.event_points),
    );
    cx.right_text(
        (215, 181),
        FONT_BODY,
        &format_decimal(stats.totals.qualification_points),
    );
    cx.right_text(
        (263, 181),
        FONT_BODY,
        &format_decimal(stats.totals.round1_points),
    );
    cx.right_text(
        (311, 181),
        FONT_BODY,
        &format_decimal(stats.totals.round2_points),
    );
    cx.text((16, 190), FONT_GOLD, "AVG");
    cx.right_text(
        (168, 190),
        FONT_BODY,
        &format_decimal(stats.totals.average_event_points),
    );
    render_jump_average(
        cx,
        stats.totals.average_qualification_points,
        stats.totals.average_qualification_distance,
        215,
    );
    render_jump_average(
        cx,
        stats.totals.average_round1_points,
        stats.totals.average_round1_distance,
        263,
    );
    render_jump_average(
        cx,
        stats.totals.average_round2_points,
        stats.totals.average_round2_distance,
        311,
    );

    render_page_hints(cx, stats.page, stats.total_pages, lang, PageHintLayout::Top);
}

fn render_history_jump(cx: &mut dyn UiCanvas, jump: Option<&EventJumpHistory>, x: i32, y: i32) {
    if let Some(jump) = jump {
        cx.right_text((x, y), FONT_BODY, &format_decimal(jump.points));
        cx.right_text(
            (x, y + 8),
            FONT_TEAL,
            &format!("{}µ", format_decimal(jump.distance)),
        );
    }
}

fn qualification_status(status: QualificationStatus) -> &'static str {
    match status {
        QualificationStatus::NotQualified => "NQ",
        QualificationStatus::Qualified => "Q",
        QualificationStatus::PreQualified => "PQ",
        QualificationStatus::LuckyLoser => "LL",
        QualificationStatus::KoSeed(_) => "KO",
        QualificationStatus::Eliminated => "OUT",
    }
}

fn render_jump_average(cx: &mut dyn UiCanvas, points: Option<f64>, distance: Option<f64>, x: i32) {
    if let (Some(points), Some(distance)) = (points, distance) {
        cx.right_text(
            (x, 190),
            FONT_TEAL,
            &format!("{}/{}", format_decimal(points), format_decimal(distance)),
        );
    }
}

fn format_distance(value: f64, value2: f64) -> String {
    let mut length = format_decimal(value);
    while length.len() < 5 {
        length.insert(0, '$');
    }
    if value2 == 0.0 {
        return format!("({length}µ)");
    }

    let mut second = format_decimal(value2);
    while second.len() < 5 {
        second.insert(0, '$');
    }
    format!("({length}-{second}µ)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::field::SortBy;
    use crate::competition::types::{CustomCupScoring, EventJumpHistory, IndividualEventHistory};

    fn custom_results(scoring: CustomCupScoring) -> Competition {
        let mut participant = Participant::computer(0, 0, "Jumper".into());
        participant.wc_points = 100;
        participant.four_hills_points = 432.5;
        let mut competition = Competition::new(CupStyle::CustomCup, vec![participant], vec![0]);
        competition.custom_cup_scoring = scoring;
        competition.phase = CompetitionPhase::SeasonComplete;
        if competition.uses_aggregate_standings() {
            competition.field.sort_field(SortBy::FourHillsPoints);
        } else {
            competition.field.sort_field(SortBy::WcPoints);
        }
        competition
    }

    #[test]
    fn custom_cup_final_results_use_selected_scoring_mode() {
        let wc_page = build_results_page(&custom_results(CustomCupScoring::WorldCupPoints), 0);
        assert_eq!(wc_page.items[0].points, 100.0);
        assert!(!wc_page.items[0].use_tenths);

        let aggregate_page =
            build_results_page(&custom_results(CustomCupScoring::AggregateJumpPoints), 0);
        assert_eq!(aggregate_page.items[0].points, 432.5);
        assert!(aggregate_page.items[0].use_tenths);
    }

    #[test]
    fn stats_page_paginates_events_and_totals_all_history() {
        let mut participant = Participant::computer(0, 0, "Human".into());
        participant.is_computer = false;
        participant.event_history = (0..10)
            .map(|event_index| IndividualEventHistory {
                event_index,
                hill_idx: event_index,
                qualification: Some(EventJumpHistory {
                    distance: 100.0 + event_index as f64,
                    points: 100.0,
                }),
                qualification_status: QualificationStatus::Qualified,
                round1: Some(EventJumpHistory {
                    distance: 110.0,
                    points: 120.0,
                }),
                round2: Some(EventJumpHistory {
                    distance: 115.0,
                    points: 130.0,
                }),
                event_points: Some(250.0),
                placing: Some(event_index + 1),
                wc_points_awarded: 10,
                running_rank: 2,
                status: QualificationStatus::Qualified,
                reason: None,
            })
            .collect();
        let competition =
            Competition::new(CupStyle::WorldCup, vec![participant], (0..10).collect());

        assert_eq!(stats_total_pages(&competition), 2);
        let first = build_stats_page(&competition, 0).unwrap();
        let second = build_stats_page(&competition, 1).unwrap();
        assert_eq!(first.events.len(), 8);
        assert_eq!(second.events.len(), 2);
        assert_eq!(second.events[0].event_index, 8);
        assert_eq!(second.totals.events, 10);
        assert_eq!(second.totals.event_points, 2500.0);
        assert_eq!(second.totals.wc_points, 100);
        assert_eq!(second.totals.average_event_points, 250.0);
        assert_eq!(second.totals.qualification_points, 1000.0);
        assert_eq!(second.totals.round1_points, 1200.0);
        assert_eq!(second.totals.round2_points, 1300.0);
        assert_eq!(second.totals.average_round2_points, Some(130.0));
        assert_eq!(second.totals.average_qualification_distance, Some(104.5));
    }

    #[test]
    fn round_one_pages_reserve_a_slot_for_the_cut_separator() {
        let participants: Vec<_> = (1..=75)
            .map(|rank| {
                let mut participant = Participant::computer(rank, rank, format!("Jumper {rank}"));
                participant.rank = rank;
                participant
            })
            .collect();
        let standings: Vec<_> = participants.iter().collect();
        let ranges = result_page_ranges(&standings, CompetitionPhase::Round1Results, 22);

        assert_eq!(ranges, vec![(0, 22), (22, 43), (43, 65), (65, 75)]);
    }

    #[test]
    fn stats_have_no_pages_without_a_human_participant() {
        let competition = Competition::new(
            CupStyle::WorldCup,
            vec![Participant::computer(0, 0, "Computer".into())],
            vec![0],
        );

        assert_eq!(stats_total_pages(&competition), 0);
        assert!(build_stats_page(&competition, 0).is_none());
    }
}
