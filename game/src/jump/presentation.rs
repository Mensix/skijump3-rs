use crate::data::records::HillRecord;
use crate::gfx::color::Rgb6;
use crate::gfx::materials;
use crate::gfx::sprites;
use crate::gfx::theme::{FILL_PURPLE, FILL_TEAL, FONT_BODY, FONT_GOLD, FONT_TEAL};
use crate::jump::frame::JumpRenderFrame;
use crate::jump::hud;
use crate::jump::types::JumpPhase;
use crate::jump::visuals::{self, JumperSpriteSpec};
use crate::text::lang::LangBase;
use crate::ui::Font;
use crate::ui::UiCanvas;
use engine::color::Rgba;

pub use crate::jump::wind::WindPosition;

pub struct JumpPresentationContext<'a> {
    pub(crate) font: &'a Font,
    pub(crate) langbase: &'a LangBase,
    pub(crate) jumper_name: &'a str,
    pub(crate) hill_name_k: &'a str,
    pub(crate) hill_record: Option<&'a HillRecord>,
    pub(crate) goal_distance: Option<f64>,
    pub(crate) wind_position: WindPosition,
    pub(crate) phase_label: &'a str,
    pub(crate) allow_gate_adjust: bool,
    pub(crate) suppress_info_panel: bool,
    pub(crate) suit_color: Rgb6,
    pub(crate) ski_color: Rgb6,
    pub(crate) is_computer: bool,
    pub(crate) team_name: &'a str,
    pub(crate) has_bib: bool,
}

pub fn render(
    cx: &mut dyn UiCanvas,
    frame: &mut JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    visuals::push_hill_layers(
        cx,
        &frame.back_layer,
        &frame.front_layer,
        std::mem::take(&mut frame.snow_pixels),
        frame.sx,
        frame.sy,
        None,
    );

    let start_light_material =
        materials::start_light_material(frame.phase == JumpPhase::Disqualified);

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
            info_panel_elements(cx, ctx);
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
        cx.sprite_with_material(
            sprites::Sprite::StartLight as u16,
            (jumper_x + 60, jumper_y - 10),
            start_light_material,
        );
    }

    visuals::push_hill_record_marker(cx, frame.hill_record_marker, frame.sx, frame.sy);
    visuals::push_goal_marker(cx, frame.goal_marker, frame.sx, frame.sy);

    if let Some((x, y)) = frame.bar_gag_position {
        cx.fill((x - frame.sx, y - frame.sy, 1, 1), FONT_GOLD);
    }

    if renders_jumper(frame.phase) {
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

    if ctx.is_computer && renders_jumper(frame.phase) {
        cx.text((jumper_x, jumper_y - 20), FONT_TEAL, "C");
    }
}

const fn renders_jumper(phase: JumpPhase) -> bool {
    matches!(
        phase,
        JumpPhase::OnBar
            | JumpPhase::Inrun
            | JumpPhase::Flight
            | JumpPhase::Landing
            | JumpPhase::Result
            | JumpPhase::Disqualified
    )
}

fn gate_info_elements(
    cx: &mut dyn UiCanvas,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    let lang = ctx.langbase;
    if ctx.allow_gate_adjust {
        let label58 = lang.tr(58);
        let label58_w = ctx.font.string_width(label58) as i32;
        cx.text((64, 19), FONT_BODY, label58);
        cx.text(
            (70 + label58_w, 19),
            FONT_GOLD,
            &format!("{}", frame.start_gate),
        );
        cx.text((67 + label58_w, 27), FONT_TEAL, "(+/-)");
    }
}

fn jumper_info_box_elements(cx: &mut dyn UiCanvas, ctx: &JumpPresentationContext<'_>) {
    let lang = ctx.langbase;
    let phase_label = if ctx.phase_label.is_empty() {
        lang.tr(51)
    } else {
        ctx.phase_label
    };
    let subline = (!ctx.team_name.is_empty()).then_some((ctx.team_name, FILL_TEAL));
    hud::push_jumper_info_box(
        cx,
        ctx.font,
        ctx.langbase,
        phase_label,
        ctx.jumper_name,
        subline,
    );
}

fn info_elements(
    cx: &mut dyn UiCanvas,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    info_panel_elements(cx, ctx);
    gate_info_elements(cx, frame, ctx);
    jumper_info_box_elements(cx, ctx);
}

fn info_panel_elements(cx: &mut dyn UiCanvas, ctx: &JumpPresentationContext<'_>) {
    if ctx.suppress_info_panel {
        return;
    }
    hud::push_hill_record_info(
        cx,
        ctx.langbase,
        ctx.hill_name_k,
        ctx.hill_record,
        ctx.goal_distance,
    );
}

fn panel_header(cx: &mut dyn UiCanvas, name: &str, color: Rgba) {
    hud::push_info_panel_frame(cx);
    cx.right_text((308, 9), color, name);
}

fn panel_distance(cx: &mut dyn UiCanvas, distance: f64) {
    cx.right_text((308, 33), FONT_TEAL, &format!("{distance:.1}m"));
}

fn result_elements(
    cx: &mut dyn UiCanvas,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    let lang = ctx.langbase;
    panel_header(cx, ctx.jumper_name, FONT_BODY);
    if !ctx.team_name.is_empty() {
        cx.right_text((302, 14), FONT_TEAL, ctx.team_name);
    }
    let style_min = *frame.style_points.iter().min().unwrap_or(&0);
    let style_max = *frame.style_points.iter().max().unwrap_or(&0);
    let first_min_idx = frame.style_points.iter().position(|&p| p == style_min);
    let first_max_idx = frame.style_points.iter().position(|&p| p == style_max);
    for (i, &point) in frame.style_points.iter().enumerate() {
        let color = if Some(i) == first_min_idx || Some(i) == first_max_idx {
            FILL_TEAL
        } else {
            FONT_TEAL
        };
        cx.right_text(
            (308 - (i as i32) * 24, 21),
            color,
            &format!("{:.1}", f64::from(point) / 10.0),
        );
    }

    if frame.is_hill_record {
        cx.text((260, 33), FONT_GOLD, "HR!");
        cx.right_text((308, 33), FONT_GOLD, &format!("{:.1}m", frame.distance));
    } else {
        panel_distance(cx, frame.distance);
    }

    cx.right_text(
        (308, 45),
        FONT_GOLD,
        &format!("{:.1}", f64::from(frame.score) / 10.0),
    );
    cx.right_text((308, 73), FONT_TEAL, lang.tr(298));
}

fn landing_elements(
    cx: &mut dyn UiCanvas,
    frame: &JumpRenderFrame,
    ctx: &JumpPresentationContext<'_>,
) {
    panel_header(cx, ctx.jumper_name, FONT_TEAL);

    panel_distance(cx, frame.distance);

    if frame.is_hill_record {
        if frame.frame_counter % 30 < 15 {
            cx.text((260, 33), FONT_GOLD, "HR!");
        }

        if let Some((x, y)) = frame.hr_shake_position {
            cx.right_text((x, y), FONT_TEAL, &format!("{:.1}m", frame.distance));
        }
    }

    for (i, &point) in frame.style_points.iter().enumerate() {
        if frame.style_revealed[i] {
            cx.right_text(
                (308 - (i as i32) * 24, 21),
                FONT_TEAL,
                &format!("{:.1}", f64::from(point) / 10.0),
            );
        }
    }
}

fn dq_elements(cx: &mut dyn UiCanvas, frame: &JumpRenderFrame, ctx: &JumpPresentationContext<'_>) {
    let lang = ctx.langbase;
    let jumper_x = frame.x - frame.sx;
    let jumper_y = frame.y - frame.sy;
    cx.sprite_with_material(
        sprites::Sprite::StartLight as u16,
        (jumper_x + 60, jumper_y - 10),
        materials::start_light_material(true),
    );
    cx.sprite(sprites::Sprite::JumperInfoBox as u16, (3, 150));
    cx.text(
        (12, 160),
        FONT_BODY,
        &format!("{} {}", ctx.jumper_name, lang.tr(79)),
    );
}

pub fn wind_elements(cx: &mut dyn UiCanvas, position: WindPosition, value: i32) {
    let x = position.x;
    let y = position.y;
    cx.fill((x + 4, y + 1, 35, 2), FILL_PURPLE);
    cx.fill((x + 21, y + 1, 1, 2), FONT_BODY);
    cx.fill((x + 21, y + 9, 1, 1), FONT_TEAL);
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
        cx.text((x + 10, y + 5), FONT_TEAL, "-");
    }
    let mut chars = text.chars();
    if let Some(ones) = chars.next() {
        cx.text((x + 15, y + 5), FONT_TEAL, &ones.to_string());
    }
    if let Some(tenths) = text.chars().nth(2) {
        cx.text((x + 24, y + 5), FONT_TEAL, &tenths.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landing_and_result_keep_the_jumper_visible() {
        assert!(renders_jumper(JumpPhase::Landing));
        assert!(renders_jumper(JumpPhase::Result));
        assert!(!renders_jumper(JumpPhase::Info));
    }

    #[test]
    fn computer_marker_matches_visible_jumper_phases() {
        assert!(renders_jumper(JumpPhase::OnBar));
        assert!(renders_jumper(JumpPhase::Inrun));
        assert!(renders_jumper(JumpPhase::Flight));
        assert!(renders_jumper(JumpPhase::Landing));
        assert!(renders_jumper(JumpPhase::Result));
        assert!(renders_jumper(JumpPhase::Disqualified));
        assert!(!renders_jumper(JumpPhase::Info));
    }
}
