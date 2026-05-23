use crate::competition::machine::Competition;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};
use crate::components::screen::{new_screen, page_hints};
use crate::gfx::palette::{FONT_DEFAULT, FONT_GREET, FONT_HEADER};
use crate::store::ResourcesRef;
use crate::text::format::format_tenths;
use engine::ui::Element;

pub const QUALIFICATION_ITEMS_PER_PAGE: usize = 25;

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

const OTHER_NAME: u8 = 241;
const OTHER_RANK: u8 = 251;
const OTHER_DISTANCE: u8 = 252;
const INJURY_COLOR: u8 = 249;

const KO_LEFT_POINTS: i32 = 40;
const KO_LEFT_NAME: i32 = 145;
const KO_RIGHT_NAME: i32 = 175;
const KO_RIGHT_POINTS: i32 = 303;
const KO_LEFT_STATUS: i32 = 12;
const KO_RIGHT_STATUS: i32 = 308;

pub struct ResultsPage {
    pub(crate) phase: CompetitionPhase,
    pub(crate) page: usize,
    pub(crate) total_pages: usize,
    pub(crate) items: Vec<ResultsEntry>,
    pub(crate) compact: bool,
    /// Rank of the last entry on the previous page (0 for page 0).
    /// Used to correctly place the --- separator at the 30→31 transition
    /// even when it spans across pages.
    pub(crate) prev_last_rank: usize,
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
    pub(crate) use_tenths: bool,
}

fn use_tenths_for_phase(phase: CompetitionPhase, style: CupStyle) -> bool {
    matches!(
        phase,
        CompetitionPhase::QualificationResults
            | CompetitionPhase::Round1Results
            | CompetitionPhase::Round2Results
            | CompetitionPhase::FourHillsStandings
    ) || (phase == CompetitionPhase::SeasonComplete
        && matches!(style, CupStyle::FourHills | CupStyle::CustomCup))
}

fn competition_results_entry_data(
    phase: CompetitionPhase,
    style: CupStyle,
    p: &Participant,
) -> (i32, i32, i32) {
    match phase {
        CompetitionPhase::QualificationResults => (p.points.unwrap_or(0), p.qual_len, 0),
        CompetitionPhase::Round1Results => (p.points.unwrap_or(0), p.round1_len, 0),
        CompetitionPhase::Round2Results => (p.points.unwrap_or(0), p.round1_len, p.round2_len),
        CompetitionPhase::FourHillsStandings => (p.four_hills_points, 0, 0),
        CompetitionPhase::WorldCupStandings => (p.wc_points, 0, 0),
        CompetitionPhase::SeasonComplete
            if matches!(style, CupStyle::FourHills | CupStyle::CustomCup) =>
        {
            (p.four_hills_points, 0, 0)
        }
        CompetitionPhase::SeasonComplete => (p.wc_points, 0, 0),
        _ => (p.points.unwrap_or(0), 0, 0),
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
    let total_pages = standings.len().div_ceil(per_page).max(1);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * per_page;
    let end = (start + per_page).min(standings.len());

    let phase = competition.phase();
    let style = competition.style();
    let use_tenths = use_tenths_for_phase(phase, style);
    let mut items = Vec::with_capacity(end - start);
    for &p in &standings[start..end] {
        let (points, dist, dist2) = competition_results_entry_data(phase, style, p);
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
        page,
        total_pages,
        items,
        compact: false,
        prev_last_rank,
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

    let phase = competition.phase();
    let style = competition.style();
    let use_tenths = use_tenths_for_phase(phase, style);
    let mut items = Vec::with_capacity(selected.len());
    for p in selected {
        let (points, dist, dist2) = competition_results_entry_data(phase, style, p);
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
        page: 0,
        total_pages: 1,
        items,
        compact: true,
        prev_last_rank: 0,
    }
}

pub fn total_pages(competition: &Competition) -> usize {
    let per_page = items_per_page(competition.phase());
    standings_for_phase(competition)
        .len()
        .div_ceil(per_page)
        .max(1)
}

fn standings_for_phase(competition: &Competition) -> Vec<&Participant> {
    let phase = competition.phase();
    if phase == CompetitionPhase::FourHillsStandings
        || phase == CompetitionPhase::SeasonComplete
            && matches!(
                competition.style(),
                CupStyle::FourHills | CupStyle::CustomCup
            )
    {
        competition
            .overall_standings()
            .into_iter()
            .filter(|p| p.four_hills_points > 0)
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
            .filter(|p| p.points.is_some())
            .collect()
    } else {
        competition.event_standings()
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

    let cup_style_str = |style: CupStyle, offset: usize| -> String {
        let cup_idx = match style {
            CupStyle::WorldCup => 0,
            CupStyle::CustomCup => 1,
            CupStyle::FourHills => 2,
            CupStyle::TeamCup => 3,
        };
        lang.lstr(offset + cup_idx).to_string()
    };

    let phase = competition.phase();
    let header = match phase {
        _ if phase == CompetitionPhase::QualificationResults => {
            format!(
                "{} {} {} {} - {}",
                lang.lstr(82),
                event,
                lang.lstr(8),
                total,
                hill_str
            )
        }
        _ if phase == CompetitionPhase::Round1Results => {
            round_header(lang.lstr(81), event, lang.lstr(8), total, &hill_str, 1)
        }
        _ if phase == CompetitionPhase::Round2Results => {
            round_header(lang.lstr(81), event, lang.lstr(8), total, &hill_str, 2)
        }
        _ if phase == CompetitionPhase::FourHillsStandings => {
            if competition.style() == CupStyle::FourHills && event >= total {
                lang.lstr(85).to_string()
            } else if competition.style() == CupStyle::FourHills {
                format!(
                    "{} {} {} {} - {}",
                    lang.lstr(84),
                    event,
                    lang.lstr(8),
                    total,
                    hill_str
                )
            } else {
                let prefix = cup_style_str(competition.style(), 27);
                format!(
                    "{} {} {} {} {}",
                    prefix,
                    lang.lstr(87),
                    event,
                    lang.lstr(8),
                    total
                )
            }
        }
        _ if phase == CompetitionPhase::WorldCupStandings => {
            let prefix = cup_style_str(competition.style(), 27);
            format!(
                "{} {} {} {} {}",
                prefix,
                lang.lstr(87),
                event,
                lang.lstr(8),
                total
            )
        }
        _ if phase == CompetitionPhase::SeasonComplete => {
            if competition.style() == CupStyle::FourHills {
                lang.lstr(85).to_string()
            } else {
                format!(
                    "{} {}",
                    lang.lstr(90),
                    cup_style_str(competition.style(), 27)
                )
            }
        }
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

fn render_results_entry(
    els: &mut Vec<Element>,
    entry: &ResultsEntry,
    phase: CompetitionPhase,
    y: i32,
    rank_x: i32,
    name_x: i32,
    points_x: i32,
    last_rank: &mut usize,
    show_extra: bool,
) {
    let (col_text, col_rank, col_dist) = if entry.is_own {
        (FONT_DEFAULT, FONT_HEADER, FONT_GREET)
    } else {
        (OTHER_NAME, OTHER_RANK, OTHER_DISTANCE)
    };

    if entry.rank != *last_rank {
        els.push(Element::text(
            format!("{}.", entry.rank),
            rank_x,
            y,
            col_rank,
            true,
        ));
    }
    *last_rank = entry.rank;

    els.push(Element::text(
        truncate_name(&entry.name),
        name_x,
        y,
        col_text,
        false,
    ));

    let points = if entry.use_tenths {
        format_tenths(entry.points)
    } else {
        entry.points.to_string()
    };
    els.push(Element::text(points, points_x, y, col_text, true));

    if show_extra && entry.distance > 0 {
        els.push(Element::text(
            format_distance(entry.distance, entry.distance2),
            COL_DISTANCE,
            y,
            col_dist,
            false,
        ));
    }

    if show_extra {
        if phase == CompetitionPhase::QualificationResults {
            match entry.qual {
                QualificationStatus::Qualified => {
                    els.push(Element::text("Q", COL_QUAL, y, col_rank, false));
                }
                QualificationStatus::PreQualified => {
                    els.push(Element::text("Q WC", COL_QUAL, y, col_dist, false));
                }
                _ => {}
            }
        } else if phase == CompetitionPhase::Round1Results && entry.rank <= 30 {
            els.push(Element::text("Q", COL_QUAL, y, col_rank, false));
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

    if page.total_pages == 1 && page.items.len() <= 20 {
        els.push(Element::text(
            resources.langbase.lstr(86),
            30,
            190,
            FONT_GREET,
            false,
        ));
    }

    let is_wc = page.phase == CompetitionPhase::FourHillsStandings
        || page.phase == CompetitionPhase::WorldCupStandings
        || page.phase == CompetitionPhase::SeasonComplete;
    // Pascal: Quali=plus7(phase0), Rounds=plus8(phase1), WC=plus8(phase3)
    let row_step = if is_wc {
        WC_ROW_STEP
    } else if page.phase.result_round_number().is_some() {
        8
    } else {
        ROW_STEP_QUALIFICATION
    };

    let mut last_rank = 0;
    if is_wc {
        // Two-column layout: y computed from column/row, no separator needed
        for (i, entry) in page.items.iter().enumerate() {
            let col = if i >= WC_COL_SPLIT { 1 } else { 0 };
            let col_off = col * WC_COL2_OFFSET;
            let y = START_Y + (i % WC_COL_SPLIT) as i32 * row_step;
            render_results_entry(
                &mut els,
                entry,
                page.phase,
                y,
                WC_RANK + col_off,
                WC_NAME + col_off,
                WC_POINTS + col_off,
                &mut last_rank,
                false,
            );
        }
    } else {
        let mut y = START_Y;
        for (i, entry) in page.items.iter().enumerate() {
            if y > 191 {
                break;
            }

            // Pascal: - - - separator between rank 30 and 31 in Round 1 results
            // sija[who]>30 && sija[prev]<=30 — the actual predecessor in the FULL list
            // For paginated display, check either previous visible item or prev page's last rank
            if !page.compact
                && page.phase == CompetitionPhase::Round1Results
                && entry.rank > 30
                && (i > 0 && page.items[i - 1].rank <= 30 || i == 0 && page.prev_last_rank <= 30)
            {
                let half = row_step / 2;
                els.push(Element::text(
                    "- - -",
                    COL_NAME,
                    y + half,
                    FONT_HEADER,
                    false,
                ));
                y += half + row_step + half;
                if y > 191 {
                    break;
                }
            }

            render_results_entry(
                &mut els,
                entry,
                page.phase,
                y,
                COL_RANK,
                COL_NAME,
                COL_POINTS,
                &mut last_rank,
                true,
            );
            y += row_step;
        }
    }

    els
}

pub fn render_ko_pairs(
    competition: &Competition,
    resources: &ResourcesRef,
    show_results: bool,
    show_cursor: bool,
) -> Vec<Element> {
    let mut els = new_screen(1);
    els.push(Element::text(
        resources.langbase.lstr(94),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));

    // Pascal: showpairs uses luett/mcluett (saved seed-pairing order).
    // For intro (QualificationResults), event_order is still in seed order.
    // For results (Round1Results), event_order has been re-sorted by points;
    // we must use the saved ko_pairing_standings to preserve original pairings.
    let standings = if show_results {
        competition.ko_pairing_standings()
    } else {
        competition.event_standings()
    };
    let count = standings.len().min(50);
    let half = count / 2;
    for pair in 0..half.min(25) {
        let y = 24 + pair as i32 * 7;
        let left = standings[half + pair];
        let right = standings[half - 1 - pair];
        render_ko_side(&mut els, left, y, true, show_results, resources);
        els.push(Element::text("vs.", 154, y, OTHER_NAME, false));
        render_ko_side(&mut els, right, y, false, show_results, resources);
    }

    // Pascal waitforkey3(305,6,ch):
    //   fontcolor(240); ewritefont(305,6,lstr(15));  → text right-aligned at (305,6)
    //   getch(306,6,243,ch,ch2) → fillbox(304,4,312,14,243)  → 9×11 box
    //   givech: fillbox(306,12,310,12,col) blinking 10/10    → 5×1 cursor
    els.push(Element::text(
        resources.langbase.lstr(15),
        305,
        6,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::fillbox(304, 4, 9, 11, 243));
    if show_cursor {
        els.push(Element::fillbox(306, 12, 5, 1, FONT_DEFAULT));
    }
    els
}

fn render_ko_side(
    els: &mut Vec<Element>,
    p: &Participant,
    y: i32,
    left: bool,
    show_results: bool,
    resources: &ResourcesRef,
) {
    let own = !p.is_computer;

    // Pascal: fontcolor override chain:
    //   Base: 241 (OTHER_NAME) for others, 240 (FONT_DEFAULT) for own
    //   Results mode qualifier colors: qual=1 (Qualified) → 251 (OTHER_RANK, golden)
    //                                    qual=2 (LuckyLoser) → 252 (OTHER_DISTANCE, turquoise)
    //   Own jumper overrides EVERYTHING to 240 (FONT_DEFAULT)
    //   Non-qualified (qual=0) keep base color (241 gray)
    let name_color = if own {
        FONT_DEFAULT
    } else if show_results {
        match p.qual {
            QualificationStatus::Qualified => OTHER_RANK,
            QualificationStatus::LuckyLoser => OTHER_DISTANCE,
            _ => OTHER_NAME,
        }
    } else {
        OTHER_NAME
    };

    // Pascal: name, points, and status all use the same fontcolor
    let element_color = name_color;

    let status = match p.qual {
        QualificationStatus::Qualified => "Q",
        QualificationStatus::LuckyLoser => "LL",
        _ => "",
    };

    // Pascal intro mode: shows (qual[who]) = KoSeed number, not rank
    let seed_str = match p.qual {
        QualificationStatus::KoSeed(n) => n.to_string(),
        _ => p.rank.to_string(),
    };

    // Pascal: plus = min(fontlen(name) + 5, 105)
    let name = p.display_name();
    let name_px_width = resources.font.string_width(name) as i32;
    let plus = (name_px_width + 5).min(105);

    if left {
        els.push(Element::text(
            truncate_name(name),
            KO_LEFT_NAME,
            y,
            element_color,
            true,
        ));
        if show_results {
            els.push(Element::text(
                format_tenths(p.points.unwrap_or(0)),
                KO_LEFT_POINTS,
                y,
                element_color,
                true,
            ));
            els.push(Element::text(
                status,
                KO_LEFT_STATUS,
                y,
                element_color,
                true,
            ));
        } else {
            els.push(Element::text(
                format!("({seed_str})"),
                KO_LEFT_NAME - plus,
                y,
                element_color,
                true,
            ));
        }
    } else {
        els.push(Element::text(
            truncate_name(name),
            KO_RIGHT_NAME,
            y,
            element_color,
            false,
        ));
        if show_results {
            els.push(Element::text(
                format_tenths(p.points.unwrap_or(0)),
                KO_RIGHT_POINTS,
                y,
                element_color,
                true,
            ));
            els.push(Element::text(
                status,
                KO_RIGHT_STATUS,
                y,
                element_color,
                false,
            ));
        } else {
            els.push(Element::text(
                format!("({seed_str})"),
                KO_RIGHT_NAME + plus,
                y,
                element_color,
                false,
            ));
        }
    }
}

pub fn render_stats_page(
    competition: &Competition,
    resources: &ResourcesRef,
    page: usize,
) -> Vec<Element> {
    let mut humans: Vec<_> = competition
        .overall_standings()
        .into_iter()
        .filter(|p| !p.is_computer)
        .collect();
    if humans.is_empty() {
        humans = competition
            .overall_standings()
            .into_iter()
            .take(1)
            .collect();
    }
    let idx = page.min(humans.len().saturating_sub(1));
    let Some(player) = humans.get(idx) else {
        return new_screen(1);
    };

    let mut els = new_screen(1);
    els.push(Element::text(
        resources.langbase.lstr(89),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));
    els.push(Element::text(
        player.display_name(),
        36 + resources.langbase.lstr(89).len() as i32 * 6,
        6,
        FONT_DEFAULT,
        false,
    ));
    els.push(Element::text(
        resources.langbase.lstr(106),
        16,
        23,
        FONT_GREET,
        false,
    ));
    els.push(Element::text(
        resources.langbase.lstr(108),
        70,
        23,
        FONT_GREET,
        true,
    ));
    els.push(Element::text(
        resources.langbase.lstr(109),
        90,
        23,
        FONT_GREET,
        true,
    ));
    els.push(Element::text(
        resources.langbase.lstr(98),
        110,
        23,
        FONT_GREET,
        true,
    ));
    els.push(Element::text(
        resources.langbase.lstr(97),
        140,
        23,
        FONT_GREET,
        true,
    ));
    els.push(Element::text("R 1", 170, 23, FONT_GREET, true));
    els.push(Element::text("R 2", 268, 23, FONT_GREET, true));

    let y = 37;
    let hill_name = resources
        .hills
        .hill(competition.current_hill())
        .map(|h| format!("{} {}", h.name.chars().take(3).collect::<String>(), h.kr))
        .unwrap_or_default();
    els.push(Element::text(
        format!("{}.", competition.current_event + 1),
        15,
        y,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::text(hill_name, 16, y, FONT_DEFAULT, false));
    els.push(Element::text(
        format!("{}.", player.rank),
        70,
        y,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::text(
        player.wc_points.to_string(),
        90,
        y,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::text(
        format!("{}.", player.rank),
        110,
        y,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::text(
        format_tenths(player.points.unwrap_or(0)),
        140,
        y,
        FONT_DEFAULT,
        true,
    ));
    if player.round1_len > 0 {
        els.push(Element::text(
            format_tenths(player.points.unwrap_or(0) - player.round2_len),
            170,
            y,
            FONT_DEFAULT,
            true,
        ));
        els.push(Element::text(
            format!("({}µ)", format_tenths(player.round1_len)),
            210,
            y,
            FONT_GREET,
            true,
        ));
    }
    if player.round2_len > 0 {
        els.push(Element::text(
            format!("({}µ)", format_tenths(player.round2_len)),
            308,
            y,
            FONT_GREET,
            true,
        ));
    }
    els.extend(page_hints(
        idx,
        humans.len().max(1),
        resources.langbase.lstr(246),
        resources.langbase.lstr(247),
        resources.langbase.lstr(248),
    ));
    els
}

fn truncate_name(name: &str) -> String {
    const MAX_CHARS: usize = 20;
    if name.chars().count() <= MAX_CHARS {
        return name.to_string();
    }

    name.chars().take(MAX_CHARS).collect()
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
