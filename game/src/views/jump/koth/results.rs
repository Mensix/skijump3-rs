use crate::competition::koth::types::KothRuntime;
use crate::components::screen::page_hints;
use crate::gfx::palette::{
    BG_4HILLS, FILL_HIGHLIGHT, FILL_TURQUOISE, FONT_DEFAULT, FONT_HEADER, FONT_HELP,
};
use crate::store::ResourcesRef;
use crate::text::format::format_decimal;
use engine::color::Rgba;
use engine::ui::Element;

// Pascal column positions (columnX[1]): rank, name, points, distance, qual, extra
const COL_RANK: i32 = 24;
const COL_NAME: i32 = 32;
const COL_POINTS: i32 = 184;
const COL_DIST: i32 = 199;
const COL_EXTRA: i32 = 275;
const START_Y: i32 = 23;
const ROW_STEP: i32 = 8;
const ITEMS_PER_PAGE: usize = 22;

const KOTH_BG: Rgba = BG_4HILLS;

/// One entry in the KOTH results list.
pub struct KothEntry {
    pub rank: usize,
    pub name: String,
    pub points: f64,
    pub dist1: f64,
    pub dist2: f64,
    pub is_human: bool,
    pub is_eliminated: bool,
    pub is_last_eliminated: bool,
    pub is_king: bool,
}

/// Paginated KOTH results data.
pub struct KothPage {
    pub page: usize,
    pub total_pages: usize,
    pub items: Vec<KothEntry>,
    pub title: String,
    pub remaining: usize,
}

/// Build a sorted list of entries matching Pascal kothjarj order.
fn build_entries(c: &KothRuntime) -> (Vec<KothEntry>, usize, bool) {
    let mut idx_sorted: Vec<usize> = (0..c.participants.len()).collect();
    // Pascal kothjarj: alive sorted by points descending,
    // eliminated sorted by elimination order (later = higher)
    // We sort: alive first (desc points), then eliminated (desc elimination round)
    idx_sorted.sort_by(|&a, &b| {
        let pa = &c.participants[a];
        let pb = &c.participants[b];
        match (pa.is_alive(), pb.is_alive()) {
            (true, true) => pb.total_points.total_cmp(&pa.total_points),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => pb.eliminated_in_round.cmp(&pa.eliminated_in_round),
        }
    });

    let remaining = c.participants.iter().filter(|p| p.is_alive()).count();
    let is_final = remaining <= 1;

    let entries: Vec<KothEntry> = idx_sorted
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let p = &c.participants[idx];
            let is_human = c.human_indices.contains(&idx);

            // Get distances from current elimination round jumps
            let (d1, d2) = if p.jumps.is_empty() {
                (0.0, 0.0)
            } else {
                let round_jumps: Vec<f64> = p
                    .jumps
                    .iter()
                    .filter(|j| j.elimination_round == c.current_elimination_round)
                    .map(|j| j.distance)
                    .collect();
                match round_jumps.len() {
                    0 => (0.0, 0.0),
                    1 => (round_jumps[0], 0.0),
                    _ => (round_jumps[0], round_jumps[1]),
                }
            };

            let is_last_eliminated =
                !p.is_alive() && p.eliminated_in_round == c.current_elimination_round;
            let is_king = is_final && pos == 0;

            let cname = if p.competitor.real_name.is_empty() {
                &p.competitor.name
            } else {
                &p.competitor.real_name
            };

            KothEntry {
                rank: pos + 1,
                name: cname.to_string(),
                points: p.total_points,
                dist1: d1,
                dist2: d2,
                is_human,
                is_eliminated: !p.is_alive(),
                is_last_eliminated,
                is_king,
            }
        })
        .collect();

    (entries, remaining, is_final)
}

/// Render the KOTH results list (Pascal kothlista).
pub fn render(resources: &ResourcesRef, store: &crate::store::StoreRef) -> Vec<Element> {
    store
        .with_active(|active| {
            let c = active.koth_runtime()?;
            let (entries, remaining, _is_final) = build_entries(c);
            let total_pages = (entries.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
            let page = 1;

            let title = if remaining <= 1 {
                "KING OF THE HILL!"
            } else {
                "KING OF THE HILL"
            };

            Some(render_page(
                &KothPage {
                    page,
                    total_pages,
                    items: entries,
                    title: title.to_string(),
                    remaining,
                },
                resources,
            ))
        })
        .flatten()
        .unwrap_or_default()
}

fn render_page(page: &KothPage, resources: &ResourcesRef) -> Vec<Element> {
    let mut els = crate::components::screen::new_screen_with_bg(1, KOTH_BG);

    // Page navigation hints
    els.extend(page_hints(
        page.page,
        page.total_pages,
        resources.langbase.lstr(246),
        resources.langbase.lstr(247),
        resources.langbase.lstr(248),
    ));

    // Title: "KING OF THE HILL" at (30, 6) - Pascal writefont(30,6,HeaderStr)
    els.push(Element::text(&page.title, 30, 6, FONT_DEFAULT, false));

    // Pascal: hint at (30, 190) when only 1 page and few entries
    if page.total_pages == 1 && page.items.len() <= 20 {
        els.push(Element::text(
            resources.langbase.lstr(86),
            30,
            190,
            FONT_HELP,
            false,
        ));
    }

    let start = (page.page - 1) * ITEMS_PER_PAGE;
    let end = start + ITEMS_PER_PAGE;
    let mut y = START_Y;
    let mut last_rank = 0usize;

    for entry in page.items.iter().skip(start).take(end) {
        if y > 180 {
            break;
        }

        // Pascal Entry colors:
        // Humans: col1=240 (FONT_DEFAULT), col2=246 (FONT_HEADER), col3=246 (setcol3)
        // Computers: col1=241 (dim), col2=251 (brighter), col3=251
        let (col_name, col_rank, col_extra) = if entry.is_human {
            (FONT_DEFAULT, FONT_HEADER, FONT_HEADER)
        } else {
            (FONT_HELP, FILL_HIGHLIGHT, FILL_TURQUOISE)
        };

        // Rank number (Pascal: ewritefont at COL_RANK, right-aligned)
        if entry.rank != last_rank {
            els.push(Element::right_text(
                format!("{}.", entry.rank),
                COL_RANK,
                y,
                col_rank,
            ));
        }
        last_rank = entry.rank;

        // Name (Pascal: writefont at COL_NAME, left-aligned, truncated to 122 chars for single column)
        let name = if entry.name.len() > 30 {
            format!("{}..", &entry.name[..28])
        } else {
            entry.name.clone()
        };
        els.push(Element::text(&name, COL_NAME, y, col_name, false));

        // Points in tenths format (Pascal: txtp, ewritefont at COL_POINTS, right-aligned)
        let pts = format_decimal(entry.points);
        els.push(Element::right_text(pts, COL_POINTS, y, col_name));

        // Distance (Pascal: (len1µ) or (len1-len2µ) at COL_DIST, left-aligned)
        if entry.dist1 > 0.0 {
            let dist_str = if entry.dist2 > 0.0 {
                format!(
                    "({}-{}µ)",
                    format_decimal(entry.dist1),
                    format_decimal(entry.dist2),
                )
            } else {
                format!("({}µ)", format_decimal(entry.dist1))
            };
            els.push(Element::text(&dist_str, COL_DIST, y, col_extra, false));
        }

        // Extra marker at COL_EXTRA
        if entry.is_king {
            // Pascal: 'K' case writes lstr(143) at column 6
            // We just write "KING" or similar
            els.push(Element::right_text(
                resources.langbase.lstr(143).to_string(),
                COL_EXTRA + 30,
                y,
                col_rank,
            ));
        }

        y += ROW_STEP;
    }

    els
}
