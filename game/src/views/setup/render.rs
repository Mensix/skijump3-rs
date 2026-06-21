use crate::components::modal::alert_prompt;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::store::GameState;
use engine::oxide::PaintCx;

use super::state::{hex_char, key_name, wind_place_name, SetupModal};
use super::view::SetupView;

pub(crate) fn paint_content(view: &SetupView, state: &GameState, cx: &mut PaintCx<'_>) {
    let lang = view.langbase();
    if let Some(SetupModal::ConfigureKeys { selected, capture }) = view.modal {
        render_configure_keys(view, state, cx, selected, capture);
        return;
    }
    if let Some(SetupModal::HillGoals(selected)) = view.modal {
        render_hill_goals(view, state, cx, selected);
        return;
    }

    render_screen(view, state, cx);

    match view.modal {
        Some(SetupModal::WindPlace(pos)) => {
            cx.fill((54, 19, 222, 162), FILL_PURPLE);
            cx.fill((55, 20, 220, 160), BG_PURPLE);
            cx.text((75, 30), FONT_GOLD, lang.tr(221));

            let winds = 10;
            for apu in 0..=winds {
                let yy = ((apu + 1) as i32) * 10 + 34;
                let name = wind_place_name(view.langbase(), apu);
                cx.right_text((85, yy), FONT_GOLD, format!("{}.", apu + 1));
                cx.text((90, yy), FONT_BODY, name);
            }

            let yy_exit = (winds * 10 + 34 + 20) as i32;
            cx.right_text((85, yy_exit), FONT_GOLD, "0.");
            cx.text((90, yy_exit), FONT_BODY, lang.tr(154));
            let stroke_y = if pos < winds {
                44 - 3 + (pos as i32) * 10
            } else {
                yy_exit - 3
            };
            cx.stroke((70 - 6, stroke_y, 140 + 1, 10 + 1), FONT_BODY);
        }
        Some(SetupModal::SeeComps(idx)) => {
            cx.fill((74, 79, 172, 54), FILL_PURPLE);
            cx.fill((75, 80, 170, 52), BG_PURPLE);
            cx.text((85, 85), FONT_BODY, lang.tr(220));
            cx.text((85, 95), FONT_GRAY, lang.tr(150));
            let opts = super::actions::seecomp_options(view, state);
            let (val, display) = opts
                .get(idx)
                .map(|(v, l)| (*v, l.as_str()))
                .unwrap_or((0, "?"));
            cx.fill((85, 105, 235 - 85 + 1, 125 - 105 + 1), FILL_GRAY);
            let color = if val >= 235 { FONT_BODY } else { FONT_GOLD };
            cx.text((95, 112), color, display);
        }
        Some(SetupModal::ConfirmReset(kind)) => {
            let label = if kind == 1 {
                lang.tr(190)
            } else {
                lang.tr(191)
            };
            alert_prompt(
                cx,
                format!("{} {}", label, lang.tr(192)),
                lang.tr(193),
                false,
            );
        }
        Some(SetupModal::LanguagePicker(sel)) => {
            cx.fill((74, 41, 172, 145), FILL_PURPLE);
            cx.fill((75, 42, 170, 143), BG_PURPLE);
            cx.text((100, 50), FONT_BODY, "PLEASE CHOOSE A LANGUAGE:");
            for (i, lang) in view.langbase().languages().iter().enumerate() {
                let yy = ((i + 1) as i32) * 8 + 55;
                cx.center_text((155, yy), FONT_GOLD, &lang.name);
            }
            cx.stroke(
                (112 - 6, 64 - 3 + (sel as i32) * 8, 100 + 1, 8 + 1),
                FONT_BODY,
            );
        }
        Some(SetupModal::ConfigureKeys { .. }) => {}
        Some(SetupModal::HillGoals(_)) => {}
        Some(SetupModal::NameSetInput) => {
            cx.fill((253, 68, 9, 11), FILL_GRAY);
            let cursor_on = view.cursor_blink.visible(11, 10);
            if cursor_on {
                cx.fill((255, 76, 5, 1), FONT_BODY);
            }
        }
        None => {}
    }
}

fn render_hill_goals(view: &SetupView, state: &GameState, cx: &mut PaintCx<'_>, selected: usize) {
    let lang = view.langbase();
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
    cx.text((30, 6), FONT_BODY, lang.tr(200));
    cx.text((40, 13), FONT_GRAY, lang.tr(243));
    cx.text((24, 23), FONT_GOLD, lang.tr(106));
    cx.right_text((200, 23), FONT_GOLD, lang.tr(242));
    cx.right_text((250, 23), FONT_GOLD, "K");
    cx.right_text((300, 23), FONT_GOLD, "HR");
    let hill_count = view.resources.hills.len().min(20);
    for idx in 0..hill_count {
        let y = (idx as i32 + 1) * 8 + 24;
        let goal_color = if idx == selected {
            FONT_GOLD
        } else {
            FONT_BODY
        };
        if let Some(hill) = view.resources.hills.hill(idx) {
            cx.right_text((18, y), FONT_GOLD, format!("{}.", idx + 1));
            cx.text((24, y), FONT_BODY, &hill.name);
            let goal = state.records.hill_goals.get(idx).copied().unwrap_or(0.0);
            cx.right_text((200, y), goal_color, format_distance(goal));
            cx.right_text((250, y), FONT_TEAL, format_distance(hill.kr as f64));
            let record = state.records.hill_records.get(idx).map_or(0.0, |r| r.len);
            cx.right_text((300, y), FONT_TEAL, format_distance(record));
        }
    }

    let exit_y = (hill_count as i32 + 1) * 8 + 24;
    let exit_color = if selected >= hill_count {
        FONT_GOLD
    } else {
        FONT_BODY
    };
    cx.right_text((200, exit_y), exit_color, lang.tr(154));
    let selected_y = if selected < hill_count {
        (selected as i32 + 1) * 8 + 22
    } else {
        exit_y - 2
    };
    cx.stroke((168, selected_y, 34, 9), FONT_BODY);
}

fn format_distance(value: f64) -> String {
    if value <= 0.0 {
        "-".to_string()
    } else {
        format!("{value:.1}")
    }
}

fn render_configure_keys(
    view: &SetupView,
    state: &GameState,
    cx: &mut PaintCx<'_>,
    selected: usize,
    capture: Option<usize>,
) {
    let lang = view.langbase();
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
    cx.text((30, 6), FONT_BODY, lang.tr(199));

    let keys = [
        state.config.key_up,
        state.config.key_right,
        state.config.key_left,
        state.config.key_telemark,
        state.config.key_replay,
    ];

    let x = 25;
    let mut y = 0;
    for temp in 0..6 {
        let item = temp;
        y = (temp as i32) * 10 + 40;
        cx.right_text((x, y), FONT_GOLD, format!("{}.", temp + 1));
        cx.text((x + 10, y), FONT_BODY, lang.tr(temp + 331));
        if temp < 5 {
            let key_text = if capture == Some(item) {
                "".to_string()
            } else {
                key_name(keys[item], view.langbase())
            };
            if capture == Some(item) {
                cx.fill((180, y - 2, 140, 10), FILL_GRAY);
                cx.pattern_fill((180, y - 2, 140, 10), BG_PURPLE);
                cx.fill((183, y - 2, 9, 11), FILL_GRAY);
                if view.cursor_blink.visible(11, 10) {
                    cx.fill((185, y + 6, 5, 1), FONT_BODY);
                }
            }
            cx.text((x + 160, y), FONT_GOLD, key_text);
        }
    }

    y += 20;
    cx.right_text((x, y), FONT_GOLD, "0.");
    cx.text((x + 10, y), FONT_BODY, lang.tr(337));

    if capture.is_none() {
        let by = if selected < 6 {
            40 - 3 + (selected as i32) * 10
        } else {
            y - 3
        };
        cx.stroke((35 - 6, by, 150 + 1, 10 + 1), FONT_BODY);
    }
}

fn render_screen(view: &SetupView, state: &GameState, cx: &mut PaintCx<'_>) {
    let lang = view.langbase();
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);

    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let title_id = match view.screen {
        0 => 175,
        1 => 176,
        2 => 177,
        3 => 178,
        _ => return,
    };
    cx.text((30, 6), FONT_BODY, lang.tr(title_id));

    let screen = view.screen;
    let entries = view.menu.item_count();

    for temp in 0..=entries {
        let value_str = if temp > 0 {
            match (screen, temp - 1) {
                (1, 0) => {
                    let info = view.langbase().languages();
                    let idx = state.config.language;
                    if idx >= 0 && (idx as usize) < info.len() {
                        info[idx as usize].name.clone()
                    } else {
                        "Undecided".to_string()
                    }
                }
                (1, 1) => {
                    if state.config.sound_effects != 0 {
                        lang.tr(6).to_string()
                    } else {
                        lang.tr(7).to_string()
                    }
                }
                (1, 2) => {
                    if state.config.graphics_detail == 0 {
                        lang.tr(13).to_string()
                    } else {
                        lang.tr(14).to_string()
                    }
                }
                (1, 3) => {
                    let n = state.config.name_set_index;
                    let hint = view.resources.namesets.title_for_config(n);
                    cx.text((40, 78), FONT_GRAY, format!("*** {hint} ***"));
                    format!("{n}")
                }
                (2, 0) => {
                    if state.config.training_rounds == 0 {
                        lang.tr(9).to_string()
                    } else {
                        view.langbase()
                            .tr(state.config.training_rounds as usize)
                            .to_string()
                    }
                }
                (2, 1) => {
                    if state.config.extra_statistics != 0 {
                        lang.tr(180).to_string()
                    } else {
                        lang.tr(185).to_string()
                    }
                }
                (2, 2) => {
                    if state.config.event_gap != 0 {
                        lang.tr(181).to_string()
                    } else {
                        lang.tr(186).to_string()
                    }
                }
                (2, 3) => {
                    if state.config.wc_gap != 0 {
                        lang.tr(181).to_string()
                    } else {
                        lang.tr(186).to_string()
                    }
                }
                (2, 4) => {
                    if state.config.compact_results != 0 {
                        lang.tr(182).to_string()
                    } else {
                        lang.tr(187).to_string()
                    }
                }
                (2, 5) => {
                    if state.config.invisible_back != 0 {
                        lang.tr(183).to_string()
                    } else {
                        lang.tr(188).to_string()
                    }
                }
                (2, 6) => {
                    if state.config.auto_hill_record_replay != 0 {
                        lang.tr(182).to_string()
                    } else {
                        lang.tr(185).to_string()
                    }
                }
                (2, 7) => {
                    if state.config.goals_enabled != 0 {
                        lang.tr(180).to_string()
                    } else {
                        lang.tr(186).to_string()
                    }
                }
                (2, 8) => {
                    if state.config.visible_computers >= 235 {
                        view.langbase()
                            .tr(state.config.visible_computers as usize)
                            .to_string()
                    } else {
                        format!("#{}", state.config.visible_computers)
                    }
                }
                (2, 9) => wind_place_name(view.langbase(), state.config.wind_position as usize),
                (2, 10) => {
                    if state.config.ko_system != 0 {
                        lang.tr(182).to_string()
                    } else {
                        lang.tr(185).to_string()
                    }
                }
                (3, 0) => {
                    if state.config.computer_hill_records != 0 {
                        lang.tr(183).to_string()
                    } else {
                        lang.tr(187).to_string()
                    }
                }
                (3, 1) => {
                    if state.config.unique_computer_names != 0 {
                        lang.tr(185).to_string()
                    } else {
                        lang.tr(180).to_string()
                    }
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };
        setup_item(view, cx, temp, entries, &value_str);
    }

    if view.modal.is_none() {
        let sel = view.menu.selected();
        if sel <= entries {
            let by = if sel < entries {
                40 - 3 + (sel as i32) * 10
            } else {
                (entries as i32) * 10 + 50 - 3
            };
            cx.stroke((35 - 6, by, 221 + 1, 10 + 1), FONT_BODY);
        }
    }
}

fn setup_item(
    view: &SetupView,
    cx: &mut PaintCx<'_>,
    index: usize,
    entries: usize,
    value_str: &str,
) {
    let lang = view.langbase();
    let xx = 25;
    let yy = if index == 0 {
        (entries as i32) * 10 + 50
    } else {
        (index as i32) * 10 + 30
    };

    let row_label = format!("{}.", hex_char(index));
    cx.right_text((xx, yy), FONT_GOLD, row_label);

    let label_id = match (view.screen, index) {
        (0, 0) => 195,
        (0, 1) => 196,
        (0, 2) => 197,
        (0, 3) => 198,
        (0, 4) => 199,
        (0, 5) => 200,
        (0, 6) => 201,
        (1, 0) => 203,
        (1, 1) => 204,
        (1, 2) => 205,
        (1, 3) => 206,
        (1, 4) => 207,
        (2, 0) => 211,
        (2, 1) => 212,
        (2, 2) => 213,
        (2, 3) => 214,
        (2, 4) => 215,
        (2, 5) => 216,
        (2, 6) => 217,
        (2, 7) => 218,
        (2, 8) => 219,
        (2, 9) => 220,
        (2, 10) => 221,
        (2, 11) => 222,
        (3, 0) => 225,
        (3, 1) => 226,
        (3, 2) => 227,
        (3, 3) => 228,
        (3, 4) => 229,
        (3, 5) => 230,
        _ => return,
    };

    cx.text((35, yy), FONT_BODY, lang.tr(label_id));

    if !value_str.is_empty() {
        cx.text((255, yy), FONT_GOLD, value_str);
    }
}
