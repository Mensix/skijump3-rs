use crate::data::records::HillRecord;
use crate::gfx::palette::{self, FILL_BORDER, FILL_TURQUOISE, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::hud;
use crate::jump::types::JumpPhase;
use crate::jump::visuals::{self, JumperSpriteSpec};
use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::ui::{Element, Font};

const FONT_DIM_TURQUOISE: Rgba = Rgba::from_rgb6(0, 47, 52);

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
    pub(crate) suppress_info_panel: bool,
    pub(crate) suit_color: usize,
    pub(crate) ski_color: usize,
    pub(crate) team_name: &'a str,
}

pub fn elements(frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) -> Vec<Element> {
    let mut els = Vec::new();
    visuals::push_viewport(&mut els, &frame.viewport);

    let start_light_recolor = palette::start_light_recolor(frame.phase == JumpPhase::Disqualified);

    match frame.phase {
        JumpPhase::Info => {
            info_elements(&mut els, frame, ctx);
        }
        JumpPhase::Result => {
            result_elements(&mut els, frame, ctx);
        }
        JumpPhase::Landing => landing_elements(&mut els, frame, ctx),
        JumpPhase::Flight => {}
        JumpPhase::OnBar => {
            info_panel_elements(&mut els, frame, ctx);
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
        els.push(Element::sprite_remapped(
            sprites::Sprite::StartLight as u16,
            jumper_x + 60,
            jumper_y - 10,
            start_light_recolor,
        ));
    }

    visuals::push_hill_record_marker(&mut els, frame.hill_record_marker, frame.sx, frame.sy);

    // Pascal: jumper not drawn during Info phase (only hill + info panel)
    if frame.phase != JumpPhase::Info {
        visuals::push_jumper_sprites(
            &mut els,
            JumperSpriteSpec {
                body_anim: frame.body_anim,
                ski_anim: frame.ski_anim,
                body_x: frame.body_x - frame.sx,
                body_y: frame.body_y - frame.sy - 2,
                ski_x: jumper_x,
                ski_y: jumper_y - 1,
                suit_color: ctx.suit_color,
                ski_color: ctx.ski_color,
            },
        );
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
    let phase_label = if ctx.phase_label.is_empty() {
        ctx.langbase.lstr(51)
    } else {
        ctx.phase_label
    };
    let subline = (!ctx.team_name.is_empty()).then_some((ctx.team_name, FILL_TURQUOISE));
    hud::push_jumper_info_box(
        els,
        ctx.font,
        ctx.langbase,
        phase_label,
        ctx.jumper_name,
        subline,
    );
}

fn info_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    info_panel_elements(els, frame, ctx);
    gate_info_elements(els, frame, ctx);
    jumper_info_box_elements(els, frame, ctx);
}

/// Draw the right-side InfoPanel sprite and its default content.
fn info_panel_elements(
    els: &mut Vec<Element>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    if ctx.suppress_info_panel {
        return;
    }
    if frame.phase == JumpPhase::OnBar {
        hud::push_keymap(els, ctx.langbase);
    } else {
        hud::push_hill_record_info(els, ctx.langbase, ctx.hill_name_k, ctx.hill_record);
    }
}

fn panel_header(els: &mut Vec<Element>, name: &str, color: Rgba) {
    hud::push_info_panel_frame(els);
    els.push(Element::right_text(name, 308, 9, color));
}

fn panel_distance(els: &mut Vec<Element>, distance: i32) {
    els.push(Element::right_text(
        format!("{:.1}m", f64::from(distance) / 10.0),
        308,
        33,
        FONT_GREET,
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
    els.push(Element::sprite_remapped(
        sprites::Sprite::StartLight as u16,
        jumper_x + 60,
        jumper_y - 10,
        palette::start_light_recolor(true),
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
    els.push(Element::fillbox(x + 4, y + 1, 35, 2, FILL_BORDER));
    els.push(Element::fillbox(x + 21, y + 1, 1, 2, FONT_DEFAULT));
    els.push(Element::fillbox(x + 21, y + 9, 1, 1, FONT_GREET));
    if value > 0 {
        els.push(Element::fillbox(
            x + 22,
            y + 1,
            value / 3 + 1,
            2,
            Rgba::from_rgb6(56, 13, 13),
        ));
    }
    if value < 0 {
        let w = (-value) / 3 + 1;
        els.push(Element::fillbox(
            x + 21 - w,
            y + 1,
            w,
            2,
            Rgba::from_rgb6(13, 53, 13),
        ));
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
