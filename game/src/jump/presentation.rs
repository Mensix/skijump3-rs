use crate::data::records::HillRecord;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::gfx::sprites;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::types::JumpPhase;
use crate::parsers::langbase::LangBase;
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Font, ImageRegion};
use std::rc::Rc;

const FONT_DIM_TURQUOISE: u8 = 252;

pub use crate::jump::wind::WindPosition;

pub struct JumpPresentationContext<'a> {
    pub(crate) font: &'a Font,
    pub(crate) langbase: &'a LangBase,
    pub(crate) jumper_name: &'a str,
    pub(crate) hill_name_k: &'a str,
    pub(crate) hill_record: Option<&'a HillRecord>,
    pub(crate) wind_position: WindPosition,
    pub(crate) phase_label: &'a str,
    pub(crate) allow_gate_adjust: bool,
}

pub fn elements(frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) -> Vec<Element> {
    let mut els = vec![Element::image_region(ImageRegion {
        pixels: Rc::clone(&frame.viewport),
        src_w: WIDTH,
        src_h: HEIGHT,
        src_x: 0,
        src_y: 0,
        dst_x: 0,
        dst_y: 0,
        w: WIDTH,
        h: HEIGHT,
    })];

    match frame.phase {
        JumpPhase::Info => info_elements(&mut els, frame, ctx),
        JumpPhase::Result => result_elements(&mut els, frame, ctx),
        JumpPhase::Landing => landing_elements(&mut els, frame, ctx),
        JumpPhase::Flight => {}
        JumpPhase::OnBar => {
            gate_info_elements(&mut els, frame, ctx);
        }
        JumpPhase::Inrun => {}
        JumpPhase::Disqualified => dq_elements(&mut els, frame, ctx),
    }

    let jumper_x = frame.x - frame.sx;
    let jumper_y = frame.y - frame.sy;
    if frame.frame_counter < 700
        && !matches!(
            frame.phase,
            JumpPhase::Info | JumpPhase::Result | JumpPhase::Landing
        )
    {
        wind_elements(&mut els, ctx.wind_position, frame.wind_value);
    }
    if frame.phase == JumpPhase::OnBar
        && (frame.frame_counter < 350 || (frame.frame_counter % 40) > 19)
    {
        els.push(Element::sprite(
            sprites::Sprite::StartLight as u16,
            jumper_x + 60,
            jumper_y - 10,
        ));
    }

    if let Some((hr_x, hr_y)) = frame.hill_record_marker {
        els.push(Element::sprite(
            sprites::Sprite::HillRecordMarker as u16,
            hr_x - frame.sx,
            hr_y - frame.sy,
        ));
    }

    // Pascal: jumper not drawn during Info phase (only hill + info panel)
    if frame.phase != JumpPhase::Info {
        els.push(Element::sprite(
            frame.body_anim,
            frame.body_x - frame.sx,
            frame.body_y - frame.sy - 2,
        ));
        els.push(Element::sprite(frame.ski_anim, jumper_x, jumper_y - 1));
    }
    els
}

fn gate_info_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    if ctx.allow_gate_adjust {
        let label58 = ctx.langbase.lstr(58);
        let label58_w = ctx.font.string_width(label58) as i32;
        els.push(Element::text(label58, 64, 19, FONT_DEFAULT, false));
        els.push(Element::text(
            format!("{}", frame.start_gate),
            70 + label58_w,
            19,
            FONT_GOLD,
            false,
        ));
        els.push(Element::text(
            "(+/-)",
            67 + label58_w,
            27,
            FONT_GREET,
            false,
        ));
    }
}

fn jumper_info_box_elements(
    els: &mut Vec<Element>,
    _frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    els.push(Element::sprite(
        sprites::Sprite::JumperInfoBox as u16,
        3,
        150,
    ));
    let phase_label = if ctx.phase_label.is_empty() {
        ctx.langbase.lstr(51)
    } else {
        ctx.phase_label
    };
    let label56 = ctx.langbase.lstr(56);
    let label_w = ctx.font.string_width(label56) as i32;
    els.push(Element::text(phase_label, 12, 160, FONT_GREET, false));
    els.push(Element::text(label56, 12, 172, FONT_GREET, false));
    els.push(Element::text(
        ctx.jumper_name,
        12 + label_w,
        172,
        FONT_DEFAULT,
        false,
    ));
    els.push(Element::text(
        ctx.langbase.lstr(59),
        12,
        191,
        FONT_HELP,
        false,
    ));
}

fn info_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
    els.push(Element::text(ctx.hill_name_k, 308, 9, FONT_GOLD, true));
    els.push(Element::text(
        ctx.langbase.lstr(65),
        308,
        19,
        FONT_GOLD,
        true,
    ));
    if let Some(record) = ctx.hill_record {
        if record.len > 0 {
            els.push(Element::text(&record.name, 308, 29, FONT_GOLD, true));
            els.push(Element::text(
                format!("{:.1}m", record.len as f64 / 10.0),
                308,
                39,
                FONT_GOLD,
                true,
            ));
        }
    }
    gate_info_elements(els, frame, ctx);
    jumper_info_box_elements(els, frame, ctx);
}

fn panel_header(els: &mut Vec<Element>, name: &str, color: u8) {
    els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
    els.push(Element::text(name, 308, 9, color, true));
}

fn panel_distance(els: &mut Vec<Element>, distance: i32) {
    els.push(Element::text(
        format!("{:.1}m", f64::from(distance) / 10.0),
        308,
        33,
        FONT_GREET,
        true,
    ));
}

fn result_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    panel_header(els, ctx.jumper_name, FONT_DEFAULT);
    let style_min = *frame.style_points.iter().min().unwrap_or(&0);
    let style_max = *frame.style_points.iter().max().unwrap_or(&0);
    let first_min_idx = frame.style_points.iter().position(|&p| p == style_min);
    let first_max_idx = frame.style_points.iter().position(|&p| p == style_max);
    for (i, &point) in frame.style_points.iter().enumerate() {
        let color = if Some(i) == first_min_idx || Some(i) == first_max_idx {
            FONT_DIM_TURQUOISE
        } else {
            FONT_GREET
        };
        els.push(Element::text(
            format!("{:.1}", f64::from(point) / 10.0),
            308 - (i as i32) * 24,
            21,
            color,
            true,
        ));
    }
    panel_distance(els, frame.distance);
    els.push(Element::text(
        format!("{:.1}", f64::from(frame.score) / 10.0),
        308,
        45,
        FONT_GOLD,
        true,
    ));
    els.push(Element::text(
        ctx.langbase.lstr(298),
        308,
        73,
        FONT_GREET,
        true,
    ));
}

fn landing_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    panel_header(els, ctx.jumper_name, FONT_GREET);
    panel_distance(els, frame.distance);
    for (i, &point) in frame.style_points.iter().enumerate() {
        if frame.style_revealed[i] {
            els.push(Element::text(
                format!("{:.1}", f64::from(point) / 10.0),
                308 - (i as i32) * 24,
                21,
                FONT_GREET,
                true,
            ));
        }
    }
}

fn dq_elements(els: &mut Vec<Element>, frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) {
    let jumper_x = frame.x - frame.sx;
    let jumper_y = frame.y - frame.sy;
    els.push(Element::sprite(
        sprites::Sprite::StartLight as u16,
        jumper_x + 60,
        jumper_y - 10,
    ));
    els.push(Element::sprite(
        sprites::Sprite::JumperInfoBox as u16,
        3,
        150,
    ));
    els.push(Element::text(
        format!("{} {}", ctx.jumper_name, ctx.langbase.lstr(79)),
        12,
        160,
        FONT_DEFAULT,
        false,
    ));
}

pub fn wind_elements(els: &mut Vec<Element>, position: WindPosition, value: i32) {
    let x = position.x;
    let y = position.y;
    els.push(Element::fillbox(x + 4, y + 1, 35, 2, 248));
    els.push(Element::fillbox(x + 21, y + 1, 1, 2, 240));
    els.push(Element::fillbox(x + 21, y + 9, 1, 1, 247));
    if value > 0 {
        els.push(Element::fillbox(x + 22, y + 1, value / 3 + 1, 2, 236));
    }
    if value < 0 {
        let w = (-value) / 3 + 1;
        els.push(Element::fillbox(x + 21 - w, y + 1, w, 2, 237));
    }

    let text = format!("{:.1}", f64::from(value.abs()) / 10.0);
    if value < 0 {
        els.push(Element::text("-", x + 10, y + 5, FONT_GREET, false));
    }
    let mut chars = text.chars();
    if let Some(ones) = chars.next() {
        els.push(Element::text(
            ones.to_string(),
            x + 15,
            y + 5,
            FONT_GREET,
            false,
        ));
    }
    if let Some(tenths) = text.chars().nth(2) {
        els.push(Element::text(
            tenths.to_string(),
            x + 24,
            y + 5,
            FONT_GREET,
            false,
        ));
    }
}
