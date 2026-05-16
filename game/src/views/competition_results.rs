use crate::competition::machine::Competition;
use crate::competition::types::CompetitionPhase;
use crate::palette_consts::{FONT_DEFAULT, FONT_GREET, FONT_HELP, FONT_HEADER};
use crate::store::ResourcesRef;
use engine::ui::Element;

const ITEMS_PER_PAGE: usize = 22;

pub(crate) struct ResultsPage {
    pub(crate) page: usize,
    pub(crate) total_pages: usize,
    pub(crate) items: Vec<ResultsEntry>,
}

pub(crate) struct ResultsEntry {
    pub(crate) is_own: bool,
    pub(crate) rank: usize,
    pub(crate) name: String,
    pub(crate) points: i32,
    pub(crate) distance: i32,
}

pub(crate) fn build_results_page(
    competition: &Competition,
    page: usize,
) -> ResultsPage {
    let standings = competition.event_standings();
    let total_pages = (standings.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * ITEMS_PER_PAGE;
    let end = (start + ITEMS_PER_PAGE).min(standings.len());

    let mut items = Vec::with_capacity(end - start);
    for &p in &standings[start..end] {
        let dist = match competition.phase {
            CompetitionPhase::Qualification => p.qual_len,
            CompetitionPhase::Round1 => p.round1_len,
            CompetitionPhase::Round2 => p.round2_len,
            _ => 0,
        };
        items.push(ResultsEntry {
            is_own: !p.is_computer,
            rank: p.rank,
            name: p.display_name().to_string(),
            points: p.points,
            distance: dist,
        });
    }

    ResultsPage { page, total_pages, items }
}

pub(crate) fn render_header(competition: &Competition, resources: &ResourcesRef) -> Vec<Element> {
    let lang = &resources.langbase;
    let event = competition.current_event + 1;
    let total = competition.total_events().max(1);
    let hill_idx = competition.hill_order.get(competition.current_event).copied().unwrap_or(0);
    let hill_str = resources.hills.hill(hill_idx)
        .map(|h| format!("{} K{}", h.name, h.kr))
        .unwrap_or_default();

    let header = match competition.phase {
        CompetitionPhase::Qualification => {
            format!("{} {} {} {} - {}", lang.lstr(82), event, lang.lstr(8), total, hill_str)
        }
        _ => String::new(),
    };

    vec![Element::fillbox(0, 0, 320, 14, 0),
         Element::text_color(header, 30, 6, FONT_DEFAULT)]
}

pub(crate) fn render_results_page(page: &ResultsPage) -> Vec<Element> {
    let mut els = vec![Element::fillbox(0, 0, 320, 200, 0)];

    // Top-right prompts
    if page.page > 0 {
        els.push(Element::text_color_right("(-PREV", 319, 5, FONT_HELP));
    }
    let prompt = if page.page + 1 >= page.total_pages { "DONE-)" } else { "NEXT-)" };
    els.push(Element::text_color_right(prompt, 319, 13, FONT_HELP));

    let start_y: i32 = 23;
    for (i, entry) in page.items.iter().enumerate() {
        let y = start_y + i as i32 * 8;
        if y > 191 { break; }

        let (col_text, col_rank, col_dist) = if entry.is_own {
            (FONT_DEFAULT, FONT_HEADER, FONT_GREET)
        } else {
            (FONT_HELP, FONT_HELP + 10, FONT_GREET + 5)
        };

        els.push(Element::text_color(format!("{}.", entry.rank), 24, y, col_rank));
        let name = if entry.name.len() > 15 {
            format!("{}..", &entry.name[..15])
        } else {
            entry.name.clone()
        };
        els.push(Element::text_color(name, 32, y, col_text));

        let pts = format!("{:.1}", f64::from(entry.points) / 10.0);
        els.push(Element::text_color_right(pts, 184, y, col_text));

        if entry.distance > 0 {
            let dist = format!("({:.1}m)", f64::from(entry.distance) / 10.0);
            els.push(Element::text_color(dist, 199, y, col_dist));
        }
    }

    els
}
