use crate::competition::machine::Competition;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};

use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_DARKEST, BG_PURPLE, BG_RED_BRIGHT, BG_WORLDCUP, BLACK, FILL_GRAY, FILL_GOLD, FILL_TEAL,
    FONT_BODY, FONT_TEAL, FONT_GOLD, FONT_GRAY,
};
use crate::store::ResourcesRef;
use crate::text::format::{format_decimal, ordinal_dot};
use engine::color::Rgba;
use engine::oxide::PaintCx;

pub const QUALIFICATION_ITEMS_PER_PAGE: usize = 25;

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
    /// Rank of the last entry on the previous page (0 for page 0).
    /// Used to correctly place the --- separator at the 30→31 transition
    /// even when it spans across pages.
    pub(crate) prev_last_rank: usize,
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

fn use_decimal_for_phase(phase: CompetitionPhase, style: CupStyle) -> bool {
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
) -> (f64, f64, f64) {
    match phase {
        CompetitionPhase::QualificationResults => (p.points.unwrap_or(0.0), p.qual_len, 0.0),
        CompetitionPhase::Round1Results => (p.points.unwrap_or(0.0), p.round1_len, 0.0),
        CompetitionPhase::Round2Results => (p.points.unwrap_or(0.0), p.round1_len, p.round2_len),
        CompetitionPhase::FourHillsStandings => (p.four_hills_points, 0.0, 0.0),
        CompetitionPhase::WorldCupStandings => (f64::from(p.wc_points), 0.0, 0.0),
        CompetitionPhase::SeasonComplete
            if matches!(style, CupStyle::FourHills | CupStyle::CustomCup) =>
        {
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
    let total_pages = standings.len().div_ceil(per_page).max(1);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * per_page;
    let end = (start + per_page).min(standings.len());

    let phase = competition.phase();
    let style = competition.style();
    let use_tenths = use_decimal_for_phase(phase, style);
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
        style: competition.style(),
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
    let use_tenths = use_decimal_for_phase(phase, style);
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
        style: competition.style(),
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
            .filter(|p| p.points.is_some())
            .collect()
    } else {
        competition.event_standings()
    }
}

pub fn render_header(cx: &mut PaintCx<'_>, competition: &Competition, resources: &ResourcesRef) {
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

    cx.text((30, 6), FONT_BODY, header);
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
    cx: &mut PaintCx<'_>,
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
        (FONT_BODY, FONT_GOLD, FONT_TEAL)
    } else {
        (OTHER_NAME, OTHER_RANK, OTHER_DISTANCE)
    };

    if entry.rank != *last_rank {
        cx.right_text((rank_x, y), col_rank, ordinal_dot(entry.rank));
    }
    *last_rank = entry.rank;

    cx.text((name_x, y), col_text, truncate_name(&entry.name));

    if entry.use_tenths {
        cx.right_text((points_x, y), col_text, format_decimal(entry.points));
    } else {
        cx.right_text((points_x, y), col_text, format!("{:.0}", entry.points));
    }

    if show_extra && entry.distance > 0.0 {
        cx.text(
            (COL_DISTANCE, y),
            col_dist,
            format_distance(entry.distance, entry.distance2),
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
                _ => {}
            }
        } else if phase == CompetitionPhase::Round1Results && entry.rank <= 30 {
            cx.text((COL_QUAL, y), col_rank, "Q");
        }

        if entry.injury > 0 {
            cx.text(
                (COL_EXTRA, y),
                INJURY_COLOR,
                format!("INJ-{}", entry.injury.saturating_sub(1)),
            );
        }
    }
}

pub fn render_results_page(cx: &mut PaintCx<'_>, page: &ResultsPage, resources: &ResourcesRef) {
    let bg = list_background(page.phase, page.style);
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), bg);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let prev = resources.langbase.lstr(246);
    let next = resources.langbase.lstr(247);
    let end = resources.langbase.lstr(248);
    if page.page > 0 {
        cx.right_text((319, 5), FONT_GRAY, format!("(-{prev}"));
    }
    let hint_text = if page.page + 1 == page.total_pages {
        end
    } else {
        next
    };
    cx.right_text((319, 13), FONT_GRAY, format!("{hint_text}-)"));

    if page.total_pages == 1 && page.items.len() <= 20 {
        cx.text((30, 190), FONT_TEAL, resources.langbase.lstr(86));
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
            let col = i32::from(i >= WC_COL_SPLIT);
            let col_off = col * WC_COL2_OFFSET;
            let y = START_Y + (i % WC_COL_SPLIT) as i32 * row_step;
            render_results_entry(
                cx,
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
                cx.text((COL_NAME, y + half), FONT_GOLD, "- - -");
                y += half + row_step + half;
                if y > 191 {
                    break;
                }
            }

            render_results_entry(
                cx,
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
}

pub fn render_stats_page(
    cx: &mut PaintCx<'_>,
    competition: &Competition,
    resources: &ResourcesRef,
    page: usize,
) {
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
        let bg = list_background(competition.phase(), competition.style());
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        cx.pattern_fill((0, 20, 320, 180), bg);

        cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
        return;
    };

    let bg = list_background(competition.phase(), competition.style());
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), bg);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    cx.text((30, 6), FONT_BODY, resources.langbase.lstr(89));
    cx.text(
        (36 + resources.langbase.lstr(89).len() as i32 * 6, 6),
        FONT_BODY,
        player.display_name(),
    );
    cx.text((16, 23), FONT_TEAL, resources.langbase.lstr(106));
    cx.right_text((70, 23), FONT_TEAL, resources.langbase.lstr(108));
    cx.right_text((90, 23), FONT_TEAL, resources.langbase.lstr(109));
    cx.right_text((110, 23), FONT_TEAL, resources.langbase.lstr(98));
    cx.right_text((140, 23), FONT_TEAL, resources.langbase.lstr(97));
    cx.right_text((170, 23), FONT_TEAL, "R 1");
    cx.right_text((268, 23), FONT_TEAL, "R 2");

    let y = 37;
    let hill_name = resources
        .hills
        .hill(competition.current_hill())
        .map(|h| format!("{} {}", h.name.chars().take(3).collect::<String>(), h.kr))
        .unwrap_or_default();
    cx.right_text(
        (15, y),
        FONT_BODY,
        ordinal_dot(competition.current_event + 1),
    );
    cx.text((16, y), FONT_BODY, hill_name);
    cx.right_text((70, y), FONT_BODY, ordinal_dot(player.rank));
    cx.right_text((90, y), FONT_BODY, player.wc_points.to_string());
    cx.right_text((110, y), FONT_BODY, ordinal_dot(player.rank));
    cx.right_text(
        (140, y),
        FONT_BODY,
        format_decimal(player.points.unwrap_or(0.0)),
    );
    if player.round1_len > 0.0 {
        cx.right_text(
            (170, y),
            FONT_BODY,
            format_decimal(player.points.unwrap_or(0.0) - player.round2_len),
        );
        cx.right_text(
            (210, y),
            FONT_TEAL,
            format!("({}µ)", format_decimal(player.round1_len)),
        );
    }
    if player.round2_len > 0.0 {
        cx.right_text(
            (308, y),
            FONT_TEAL,
            format!("({}µ)", format_decimal(player.round2_len)),
        );
    }

    let prev = resources.langbase.lstr(246);
    let next = resources.langbase.lstr(247);
    let end = resources.langbase.lstr(248);
    if idx > 0 {
        cx.right_text((319, 5), FONT_GRAY, format!("(-{prev}"));
    }
    let hint_text = if idx + 1 == humans.len().max(1) {
        end
    } else {
        next
    };
    cx.right_text((319, 13), FONT_GRAY, format!("{hint_text}-)"));
}

fn truncate_name(name: &str) -> String {
    const MAX_CHARS: usize = 20;
    if name.chars().count() <= MAX_CHARS {
        return name.to_string();
    }

    name.chars().take(MAX_CHARS).collect()
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
