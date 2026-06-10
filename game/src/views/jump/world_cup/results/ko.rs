use super::{truncate_name, OTHER_DISTANCE, OTHER_NAME, OTHER_RANK};
use crate::competition::machine::Competition;
use crate::competition::types::{Participant, QualificationStatus};
use crate::components::prompt;
use crate::components::screen::new_screen;
use crate::gfx::palette::{BG_LEFT, FONT_DEFAULT};
use crate::store::ResourcesRef;
use crate::text::format::format_decimal;
use engine::ui::Element;

const KO_LEFT_POINTS: i32 = 40;
const KO_LEFT_NAME: i32 = 145;
const KO_RIGHT_NAME: i32 = 175;
const KO_RIGHT_POINTS: i32 = 303;
const KO_LEFT_STATUS: i32 = 12;
const KO_RIGHT_STATUS: i32 = 308;

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

    // Pascal showpairs uses luett/mcluett (saved seed-pairing order).
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

    prompt::push_wait_for_key(
        &mut els,
        &resources.langbase,
        305,
        6,
        BG_LEFT,
        FONT_DEFAULT,
        FONT_DEFAULT,
        show_cursor,
    );
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

    let element_color = name_color;
    let status = match p.qual {
        QualificationStatus::Qualified => "Q",
        QualificationStatus::LuckyLoser => "LL",
        _ => "",
    };
    let seed_str = match p.qual {
        QualificationStatus::KoSeed(n) => n.to_string(),
        _ => p.rank.to_string(),
    };

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
                format_decimal(p.points.unwrap_or(0.0)),
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
                format_decimal(p.points.unwrap_or(0.0)),
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
