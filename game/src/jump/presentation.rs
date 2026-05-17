use crate::data::records::HillRecord;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::types::JumpPhase;
use crate::gfx::palette::{FONT_GOLD, FONT_DEFAULT, FONT_GREET, FONT_HELP};
use crate::parsers::langbase::LangBase;
use crate::gfx::sprites;
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Font, ImageRegion};
use std::rc::Rc;

const FONT_DIM_TURQUOISE: u8 = 252;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindGaugePosition {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

pub struct JumpPresentationContext<'a> {
    pub(crate) font: &'a Font,
    pub(crate) langbase: &'a LangBase,
    pub(crate) jumper_name: &'a str,
    pub(crate) hill_name_k: &'a str,
    pub(crate) hill_record: Option<&'a HillRecord>,
    pub(crate) wind_position: WindGaugePosition,
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
        JumpPhase::OnBar | JumpPhase::Inrun => {}
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
        els.push(Element::sprite(sprites::START_LIGHT, jumper_x + 60, jumper_y - 10));
    }

    if let Some((hr_x, hr_y)) = frame.hill_record_marker {
        els.push(Element::sprite(sprites::HILL_RECORD_MARKER, hr_x - frame.sx, hr_y - frame.sy));
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

fn info_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    els.push(Element::sprite(sprites::INFO_PANEL, 227, 2));
    els.push(Element::sprite(sprites::JUMPER_INFO_BOX, 3, 150));
    els.push(Element::text_color_right(
        ctx.hill_name_k,
        308,
        9,
        FONT_GOLD,
    ));
    els.push(Element::text_color_right(
        ctx.langbase.lstr(65),
        308,
        19,
        FONT_GOLD,
    ));
    if let Some(record) = ctx.hill_record {
        if record.len > 0 {
            els.push(Element::text_color_right(&record.name, 308, 29, FONT_GOLD));
            els.push(Element::text_color_right(
                format!("{:.1}m", record.len as f64 / 10.0),
                308,
                39,
                FONT_GOLD,
            ));
        }
    }

    let label56 = ctx.langbase.lstr(56);
    let label_w = ctx.font.string_width(label56) as i32;
    if ctx.allow_gate_adjust {
        let label58 = ctx.langbase.lstr(58);
        let label58_w = ctx.font.string_width(label58) as i32;
        els.push(Element::text_color(label58, 64, 19, FONT_DEFAULT));
        els.push(Element::text_color(
            format!("{}", frame.start_gate),
            70 + label58_w,
            19,
            FONT_GOLD,
        ));
        els.push(Element::text_color("(+/-)", 67 + label58_w, 27, FONT_GREET));
    }
    let phase_label = if ctx.phase_label.is_empty() {
        ctx.langbase.lstr(51)
    } else {
        ctx.phase_label
    };
    els.push(Element::text_color(phase_label, 12, 160, FONT_GREET));
    els.push(Element::text_color(label56, 12, 172, FONT_GREET));
    els.push(Element::text_color(
        ctx.jumper_name,
        12 + label_w,
        172,
        FONT_DEFAULT,
    ));
    els.push(Element::text_color(
        ctx.langbase.lstr(59),
        12,
        191,
        FONT_HELP,
    ));
}

fn result_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    els.push(Element::sprite(sprites::INFO_PANEL, 227, 2));
    els.push(Element::text_color_right(
        ctx.jumper_name,
        308,
        9,
        FONT_DEFAULT,
    ));
    let style_min = *frame.style_points.iter().min().unwrap_or(&0);
    let style_max = *frame.style_points.iter().max().unwrap_or(&0);
    let mut found_min = false;
    let mut found_max = false;
    for (i, &point) in frame.style_points.iter().enumerate() {
        let color = if point == style_min && !found_min {
            found_min = true;
            FONT_DIM_TURQUOISE
        } else if point == style_max && !found_max {
            found_max = true;
            FONT_DIM_TURQUOISE
        } else {
            FONT_GREET
        };
        els.push(Element::text_color_right(
            format!("{:.1}", f64::from(point) / 10.0),
            308 - (i as i32) * 24,
            21,
            color,
        ));
    }
    els.push(Element::text_color_right(
        format!("{:.1}m", f64::from(frame.distance) / 10.0),
        308,
        33,
        FONT_GREET,
    ));
    els.push(Element::text_color_right(
        format!("{:.1}", f64::from(frame.score) / 10.0),
        308,
        45,
        FONT_GOLD,
    ));
    els.push(Element::text_color_right(
        ctx.langbase.lstr(298),
        308,
        73,
        FONT_GREET,
    ));
}

fn landing_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    els.push(Element::sprite(sprites::INFO_PANEL, 227, 2));
    els.push(Element::text_color_right(
        ctx.jumper_name,
        308,
        9,
        FONT_GREET,
    ));
    els.push(Element::text_color_right(
        format!("{:.1}m", f64::from(frame.distance) / 10.0),
        308,
        33,
        FONT_GREET,
    ));
    for (i, &point) in frame.style_points.iter().enumerate() {
        if frame.style_revealed[i] {
            els.push(Element::text_color_right(
                format!("{:.1}", f64::from(point) / 10.0),
                308 - (i as i32) * 24,
                21,
                FONT_GREET,
            ));
        }
    }
}

pub fn wind_elements(els: &mut Vec<Element>, position: WindGaugePosition, value: i32) {
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
        els.push(Element::text_color("-", x + 10, y + 5, FONT_GREET));
    }
    let mut chars = text.chars();
    if let Some(ones) = chars.next() {
        els.push(Element::text_color(
            ones.to_string(),
            x + 15,
            y + 5,
            FONT_GREET,
        ));
    }
    if let Some(tenths) = text.chars().nth(2) {
        els.push(Element::text_color(
            tenths.to_string(),
            x + 24,
            y + 5,
            FONT_GREET,
        ));
    }
}
