use crate::competition::koth::types::KothRuntime;
use crate::gfx::palette::{
    BG_4HILLS, BLACK, FILL_DIM, FILL_HIGHLIGHT, FILL_TURQUOISE, FONT_DEFAULT, FONT_HEADER,
    FONT_HELP,
};
use crate::gfx::sprites;
use crate::store::ResourcesRef;
use crate::text::format::format_decimal;
use engine::color::Rgba;
use engine::oxide::PaintCx;

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
    pub is_king: bool,
}

/// Paginated KOTH results data.
pub struct KothPage {
    pub page: usize,
    pub total_pages: usize,
    pub items: Vec<KothEntry>,
    pub title: String,
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

            let _is_last_eliminated =
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
                is_king,
            }
        })
        .collect();

    (entries, remaining, is_final)
}

/// Render the KOTH results list (Pascal kothlista).
pub fn render(cx: &mut PaintCx<'_>, resources: &ResourcesRef, store: &crate::store::StoreRef) {
    store.with_active(|active| {
        let c = active.koth_runtime()?;
        let (entries, remaining, _is_final) = build_entries(c);
        let total_pages = entries.len().div_ceil(ITEMS_PER_PAGE);
        let page = 1;

        let title = if remaining <= 1 {
            "KING OF THE HILL!"
        } else {
            "KING OF THE HILL"
        };

        let kp = KothPage {
            page,
            total_pages,
            items: entries,
            title: title.to_string(),
        };

        // new_screen_with_bg(1, KOTH_BG)
        cx.fill((0, 0, 320, 200), BLACK);
        cx.fill((0, 0, 320, 19), FILL_DIM);
        cx.fill((0, 20, 320, 180), KOTH_BG);
        cx.dither_fill(63);
        cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

        // page_hints
        if kp.page > 0 {
            cx.right_text((319, 5), FONT_HELP, format!("(-{}", resources.langbase.lstr(246)));
        }
        let hint = if kp.page + 1 == kp.total_pages {
            resources.langbase.lstr(248)
        } else {
            resources.langbase.lstr(247)
        };
        cx.right_text((319, 13), FONT_HELP, format!("{}-)", hint));

        // Title
        cx.text((30, 6), FONT_DEFAULT, &kp.title);

        // Hint when only 1 page and few entries
        if kp.total_pages == 1 && kp.items.len() <= 20 {
            cx.text((30, 190), FONT_HELP, resources.langbase.lstr(86));
        }

        let start = (kp.page - 1) * ITEMS_PER_PAGE;
        let end = start + ITEMS_PER_PAGE;
        let mut y = START_Y;
        let mut last_rank = 0usize;

        for entry in kp.items.iter().skip(start).take(end) {
            if y > 180 {
                break;
            }

            let (col_name, col_rank, col_extra) = if entry.is_human {
                (FONT_DEFAULT, FONT_HEADER, FONT_HEADER)
            } else {
                (FONT_HELP, FILL_HIGHLIGHT, FILL_TURQUOISE)
            };

            if entry.rank != last_rank {
                cx.right_text((COL_RANK, y), col_rank, format!("{}.", entry.rank));
            }
            last_rank = entry.rank;

            let name = if entry.name.len() > 30 {
                format!("{}..", &entry.name[..28])
            } else {
                entry.name.clone()
            };
            cx.text((COL_NAME, y), col_name, name);

            let pts = format_decimal(entry.points);
            cx.right_text((COL_POINTS, y), col_name, pts);

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
                cx.text((COL_DIST, y), col_extra, dist_str);
            }

            if entry.is_king {
                cx.right_text(
                    (COL_EXTRA + 30, y),
                    col_rank,
                    resources.langbase.lstr(143).to_string(),
                );
            }

            y += ROW_STEP;
        }

        Some(())
    });
}
