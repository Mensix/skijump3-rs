use super::{truncate_name, OTHER_DISTANCE, OTHER_NAME, OTHER_RANK};
use crate::competition::machine::Competition;
use crate::competition::types::{Participant, QualificationStatus};
use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY};
use crate::store::ResourcesRef;
use crate::text::format::format_decimal;
use engine::oxide::PaintCx;

const KO_LEFT_POINTS: i32 = 40;
const KO_LEFT_NAME: i32 = 145;
const KO_RIGHT_NAME: i32 = 175;
const KO_RIGHT_POINTS: i32 = 303;
const KO_LEFT_STATUS: i32 = 12;
const KO_RIGHT_STATUS: i32 = 308;

pub fn render_ko_pairs(
    cx: &mut PaintCx<'_>,
    competition: &Competition,
    resources: &ResourcesRef,
    show_results: bool,
    show_cursor: bool,
) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    cx.text((30, 6), FONT_BODY, resources.langbase.lstr(94));

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
        render_ko_side(cx, left, y, true, show_results, resources);
        cx.text((154, y), OTHER_NAME, "vs.");
        render_ko_side(cx, right, y, false, show_results, resources);
    }

    cx.right_text(
        (305, 6),
        FONT_BODY,
        resources.langbase.lstr(15).to_string(),
    );
    cx.fill((304, 4, 9, 11), BG_PURPLE);
    if show_cursor {
        cx.fill((306, 12, 5, 1), FONT_BODY);
    }
}

fn render_ko_side(
    cx: &mut PaintCx<'_>,
    p: &Participant,
    y: i32,
    left: bool,
    show_results: bool,
    resources: &ResourcesRef,
) {
    let own = !p.is_computer;

    let name_color = if own {
        FONT_BODY
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
        cx.right_text((KO_LEFT_NAME, y), element_color, truncate_name(name));
        if show_results {
            cx.right_text(
                (KO_LEFT_POINTS, y),
                element_color,
                format_decimal(p.points.unwrap_or(0.0)),
            );
            cx.right_text((KO_LEFT_STATUS, y), element_color, status);
        } else {
            cx.right_text(
                (KO_LEFT_NAME - plus, y),
                element_color,
                format!("({seed_str})"),
            );
        }
    } else {
        cx.text((KO_RIGHT_NAME, y), element_color, truncate_name(name));
        if show_results {
            cx.right_text(
                (KO_RIGHT_POINTS, y),
                element_color,
                format_decimal(p.points.unwrap_or(0.0)),
            );
            cx.text((KO_RIGHT_STATUS, y), element_color, status);
        } else {
            cx.text(
                (KO_RIGHT_NAME + plus, y),
                element_color,
                format!("({seed_str})"),
            );
        }
    }
}
