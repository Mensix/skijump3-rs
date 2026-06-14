use crate::data::records::HillRecord;
use crate::gfx::palette::{self, FILL_BORDER, FILL_TURQUOISE, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::hud;
use crate::jump::types::JumpPhase;
use crate::jump::visuals::{self, JumperSpriteSpec};
use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::oxide::Font;
use engine::oxide::PaintCx;

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
    pub(crate) show_keymap: bool,
    pub(crate) has_bib: bool,
}

pub fn render(cx: &mut PaintCx<'_>, frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) {
    visuals::push_viewport(cx, &frame.viewport);

    let start_light_recolor = palette::start_light_recolor(frame.phase == JumpPhase::Disqualified);

    match frame.phase {
        JumpPhase::Info => {
            info_elements(cx, frame, ctx);
        }
        JumpPhase::Result => {
            result_elements(cx, frame, ctx);
        }
        JumpPhase::Landing => landing_elements(cx, frame, ctx),
        JumpPhase::Flight => {}
        JumpPhase::OnBar => {
            info_panel_elements(cx, frame, ctx);
        }
        JumpPhase::Inrun => {}
        JumpPhase::Disqualified => dq_elements(cx, frame, ctx),
    }

    let jumper_x = frame.x - frame.sx;
    let jumper_y = frame.y - frame.sy;
    if frame.frame_counter < 700
        && !matches!(
            frame.phase,
            JumpPhase::Info | JumpPhase::Result | JumpPhase::Landing
        )
    {
        wind_elements(cx, ctx.wind_position, frame.wind_value);
    }
    if frame.phase == JumpPhase::OnBar
        && (frame.frame_counter < 350 || (frame.frame_counter % 40) > 19)
    {
        cx.sprite_remapped(
            sprites::Sprite::StartLight as u16,
            (jumper_x + 60, jumper_y - 10),
            start_light_recolor,
        );
    }

    visuals::push_hill_record_marker(cx, frame.hill_record_marker, frame.sx, frame.sy);

    // Pascal: jumper not drawn during Info phase (only hill + info panel)
    if frame.phase != JumpPhase::Info {
        visuals::push_jumper_sprites(
            cx,
            JumperSpriteSpec {
                body_anim: frame.body_anim,
                ski_anim: frame.ski_anim,
                body_x: frame.body_x - frame.sx,
                body_y: frame.body_y - frame.sy - 2,
                ski_x: jumper_x,
                ski_y: jumper_y - 1,
                suit_color: ctx.suit_color,
                ski_color: ctx.ski_color,
                has_bib: ctx.has_bib,
            },
        );
    }
}

fn gate_info_elements(
    cx: &mut PaintCx<'_>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    if ctx.allow_gate_adjust {
        let label58 = ctx.langbase.lstr(58);
        let label58_w = ctx.font.string_width(label58) as i32;
        cx.text((64, 19), FONT_DEFAULT, label58);
        cx.text(
            (70 + label58_w, 19),
            FONT_GOLD,
            format!("{}", frame.start_gate),
        );
        cx.text((67 + label58_w, 27), FONT_GREET, "(+/-)");
    }
}

fn jumper_info_box_elements(
    cx: &mut PaintCx<'_>,
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
        cx,
        ctx.font,
        ctx.langbase,
        phase_label,
        ctx.jumper_name,
        subline,
    );
}

fn info_elements(cx: &mut PaintCx<'_>, frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) {
    info_panel_elements(cx, frame, ctx);
    gate_info_elements(cx, frame, ctx);
    jumper_info_box_elements(cx, frame, ctx);
}

/// Draw the right-side InfoPanel sprite and its default content.
fn info_panel_elements(
    cx: &mut PaintCx<'_>,
    _frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    if ctx.suppress_info_panel {
        return;
    }
    if ctx.show_keymap {
        hud::push_keymap(cx, ctx.langbase);
    } else {
        hud::push_hill_record_info(cx, ctx.langbase, ctx.hill_name_k, ctx.hill_record);
    }
}

fn panel_header(cx: &mut PaintCx<'_>, name: &str, color: Rgba) {
    hud::push_info_panel_frame(cx);
    cx.right_text((308, 9), color, name);
}

fn panel_distance(cx: &mut PaintCx<'_>, distance: f64) {
    cx.right_text((308, 33), FONT_GREET, format!("{distance:.1}m"));
}

fn result_elements(
    cx: &mut PaintCx<'_>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    panel_header(cx, ctx.jumper_name, FONT_DEFAULT);
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
        cx.right_text(
            (308 - (i as i32) * 24, 21),
            color,
            format!("{:.1}", f64::from(point) / 10.0),
        );
    }

    if frame.is_hill_record {
        // Pascal 2649-2651: "HR!" + distance in gold
        cx.text((260, 33), FONT_GOLD, "HR!");
        cx.right_text((308, 33), FONT_GOLD, format!("{:.1}m", frame.distance));
    } else {
        panel_distance(cx, frame.distance);
    }

    cx.right_text(
        (308, 45),
        FONT_GOLD,
        format!("{:.1}", f64::from(frame.score) / 10.0),
    );
    cx.right_text((308, 73), FONT_GREET, ctx.langbase.lstr(298));
}

fn landing_elements(
    cx: &mut PaintCx<'_>,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    panel_header(cx, ctx.jumper_name, FONT_GREET);

    // Pascal 2529: distance always at (temp2,33) where temp2=308
    panel_distance(cx, frame.distance);

    if frame.is_hill_record {
        // Pascal 2533: flash "HR!" — 15 frames on, 15 frames off
        if frame.frame_counter % 30 < 15 {
            cx.text((260, 33), FONT_GOLD, "HR!");
        }
        // Pascal 2536: if random(2)=0 then ewritefont(temp2-1+random(3),32+random(3),...)
        if let Some((x, y)) = frame.hr_shake_position {
            cx.right_text((x, y), FONT_GREET, format!("{:.1}m", frame.distance));
        }
    }

    for (i, &point) in frame.style_points.iter().enumerate() {
        if frame.style_revealed[i] {
            cx.right_text(
                (308 - (i as i32) * 24, 21),
                FONT_GREET,
                format!("{:.1}", f64::from(point) / 10.0),
            );
        }
    }
}

fn dq_elements(cx: &mut PaintCx<'_>, frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) {
    let jumper_x = frame.x - frame.sx;
    let jumper_y = frame.y - frame.sy;
    cx.sprite_remapped(
        sprites::Sprite::StartLight as u16,
        (jumper_x + 60, jumper_y - 10),
        palette::start_light_recolor(true),
    );
    cx.sprite(sprites::Sprite::JumperInfoBox as u16, (3, 150));
    cx.text(
        (12, 160),
        FONT_DEFAULT,
        format!("{} {}", ctx.jumper_name, ctx.langbase.lstr(79)),
    );
}

pub fn wind_elements(cx: &mut PaintCx<'_>, position: WindPosition, value: i32) {
    let x = position.x;
    let y = position.y;
    cx.fill((x + 4, y + 1, 35, 2), FILL_BORDER);
    cx.fill((x + 21, y + 1, 1, 2), FONT_DEFAULT);
    cx.fill((x + 21, y + 9, 1, 1), FONT_GREET);
    if value > 0 {
        cx.fill(
            (x + 22, y + 1, value / 3 + 1, 2),
            Rgba::from_rgb6(56, 13, 13),
        );
    }
    if value < 0 {
        let w = (-value) / 3 + 1;
        cx.fill((x + 21 - w, y + 1, w, 2), Rgba::from_rgb6(13, 53, 13));
    }

    let text = format!("{:.1}", f64::from(value.abs()) / 10.0);
    if value < 0 {
        cx.text((x + 10, y + 5), FONT_GREET, "-");
    }
    let mut chars = text.chars();
    if let Some(ones) = chars.next() {
        cx.text((x + 15, y + 5), FONT_GREET, ones.to_string());
    }
    if let Some(tenths) = text.chars().nth(2) {
        cx.text((x + 24, y + 5), FONT_GREET, tenths.to_string());
    }
}
