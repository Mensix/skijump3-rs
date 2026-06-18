use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BG_RED, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY,
};
use engine::oxide::PaintCx;

use super::state::{hex_char, key_name, wind_place_name, SetupModal};
use super::view::SetupView;

pub(crate) fn paint_content(view: &SetupView, cx: &mut PaintCx<'_>) {
    if let Some(SetupModal::ConfigureKeys { selected, capture }) = view.modal.get() {
        render_configure_keys(view, cx, selected, capture);
        return;
    }
    if let Some(SetupModal::HillGoals(selected)) = view.modal.get() {
        render_hill_goals(view, cx, selected);
        return;
    }

    render_screen(view, cx);

    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => {
            cx.fill((54, 19, 222, 162), FILL_PURPLE);
            cx.fill((55, 20, 220, 160), BG_PURPLE);
            cx.text((75, 30), FONT_GOLD, view.langbase().lstr(221));

            let winds = 11;
            for apu1 in 1..=winds {
                let yy = (apu1 as i32) * 10 + 34;
                let name = if apu1 <= 8 {
                    wind_place_name(view.langbase(), apu1)
                } else {
                    wind_place_name(view.langbase(), apu1 + 2)
                };
                cx.right_text((85, yy), FONT_GOLD, format!("{apu1}."));
                cx.text((90, yy), FONT_BODY, name);
            }

            let yy_exit = (winds * 10 + 34 + 20) as i32;
            cx.right_text((85, yy_exit), FONT_GOLD, "0.");
            cx.text((90, yy_exit), FONT_BODY, view.langbase().lstr(154));
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
            cx.text((85, 85), FONT_BODY, view.langbase().lstr(220));
            cx.text((85, 95), FONT_GRAY, view.langbase().lstr(150));
            let opts = super::actions::seecomp_options(view);
            let (val, display) = opts
                .get(idx)
                .map(|(v, l)| (*v, l.as_str()))
                .unwrap_or((0, "?"));
            cx.fill((85, 105, 235 - 85 + 1, 125 - 105 + 1), FILL_GRAY);
            let color = if val >= 235 { FONT_BODY } else { FONT_GOLD };
            cx.text((95, 112), color, display);
        }
        Some(SetupModal::ConfirmReset(kind)) => {
            cx.fill((69, 79, 183, 53), BLACK);
            cx.fill((70, 80, 181, 51), BG_RED);
            cx.pattern_fill((70, 80, 181, 51), BG_RED);
            let label = if kind == 1 {
                view.langbase().lstr(190)
            } else {
                view.langbase().lstr(191)
            };
            cx.text(
                (80, 90),
                FONT_GOLD,
                format!("{} {}", label, view.langbase().lstr(192)),
            );
            let sure = view.langbase().lstr(193);
            cx.text((80, 110), FONT_GOLD, sure);
            let tw = view.resources.font.string_width(sure) as i32;
            let xx = 88 + tw;
            let yy = 110;
            cx.fill((xx - 2, yy - 2, 9, 11), BG_PURPLE);
            if view.cursor_blink.visible(11, 10) {
                cx.fill((xx, yy + 6, 5, 1), FONT_BODY);
            }
        }
        Some(SetupModal::LanguagePicker(sel)) => {
            let langs = &view.langbase().languages;
            cx.fill((74, 41, 172, 145), FILL_PURPLE);
            cx.fill((75, 42, 170, 143), BG_PURPLE);
            cx.text((100, 50), FONT_BODY, "PLEASE CHOOSE A LANGUAGE:");
            for (i, name) in langs.iter().enumerate() {
                let yy = ((i + 1) as i32) * 8 + 55;
                cx.center_text((155, yy), FONT_GOLD, name);
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

fn render_hill_goals(view: &SetupView, cx: &mut PaintCx<'_>, selected: usize) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
    cx.text((30, 6), FONT_BODY, view.langbase().lstr(200));
    cx.text((40, 13), FONT_GRAY, view.langbase().lstr(243));
    cx.text((24, 23), FONT_GOLD, view.langbase().lstr(106));
    cx.right_text((200, 23), FONT_GOLD, view.langbase().lstr(242));
    cx.right_text((250, 23), FONT_GOLD, "K");
    cx.right_text((300, 23), FONT_GOLD, "HR");

    let state = view.store.borrow();
    let hill_count = view.resources.hills.len().min(20);
    for idx in 0..hill_count {
        let y = (idx as i32 + 1) * 8 + 24;
        if let Some(hill) = view.resources.hills.hill(idx) {
            cx.right_text((18, y), FONT_GOLD, format!("{}.", idx + 1));
            cx.text((24, y), FONT_BODY, &hill.name);
            let goal = state.records.hill_goals.get(idx).copied().unwrap_or(0.0);
            cx.right_text((200, y), FONT_GOLD, format_distance(goal));
            cx.right_text((250, y), FONT_GRAY, format_distance(hill.kr as f64));
            let record = state.records.hill_records.get(idx).map_or(0.0, |r| r.len);
            cx.right_text((300, y), FONT_GRAY, format_distance(record));
        }
    }

    let exit_y = (hill_count as i32 + 1) * 8 + 24;
    cx.right_text((200, exit_y), FONT_BODY, view.langbase().lstr(154));
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
    cx: &mut PaintCx<'_>,
    selected: usize,
    capture: Option<usize>,
) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
    cx.text((30, 6), FONT_BODY, view.langbase().lstr(199));

    let cfg = view.config();
    let keys = [
        cfg.key_up,
        cfg.key_right,
        cfg.key_left,
        cfg.key_telemark,
        cfg.key_replay,
    ];
    drop(cfg);

    let x = 25;
    let mut y = 0;
    for temp in 1..=6 {
        let item = temp - 1;
        y = (temp as i32) * 10 + 30;
        cx.right_text((x, y), FONT_GOLD, format!("{temp}."));
        cx.text((x + 10, y), FONT_BODY, view.langbase().lstr(temp + 330));
        if temp < 6 {
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
    cx.text((x + 10, y), FONT_BODY, view.langbase().lstr(337));

    if capture.is_none() {
        let by = if selected < 6 {
            40 - 3 + (selected as i32) * 10
        } else {
            y - 3
        };
        cx.stroke((35 - 6, by, 150 + 1, 10 + 1), FONT_BODY);
    }
}

fn render_screen(view: &SetupView, cx: &mut PaintCx<'_>) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);

    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let title_id = match view.screen.get() {
        0 => 175,
        1 => 176,
        2 => 177,
        3 => 178,
        _ => return,
    };
    cx.text((30, 6), FONT_BODY, view.langbase().lstr(title_id));

    let cfg = view.config();
    let screen = view.screen.get();
    let entries = view.menu.item_count();

    for temp in 0..=entries {
        let value_str = if temp > 0 {
            match (screen, temp - 1) {
                (1, 0) => {
                    let ln = cfg.languagenumber;
                    let all = &view.langbase().languages;
                    if ln >= 0 && (ln as usize) < all.len() {
                        all[ln as usize].clone()
                    } else {
                        "Undecided".to_string()
                    }
                }
                (1, 1) => {
                    if cfg.beeppi != 0 {
                        view.langbase().lstr(6).to_string()
                    } else {
                        view.langbase().lstr(7).to_string()
                    }
                }
                (1, 2) => {
                    if cfg.gdetail == 0 {
                        view.langbase().lstr(13).to_string()
                    } else {
                        view.langbase().lstr(14).to_string()
                    }
                }
                (1, 3) => {
                    let n = cfg.namenumber;
                    let hint = view.resources.namesets.title_for_config(n);
                    cx.text((40, 78), FONT_GRAY, format!("*** {hint} ***"));
                    format!("{n}")
                }
                (2, 0) => {
                    if cfg.trainrounds == 0 {
                        view.langbase().lstr(9).to_string()
                    } else {
                        view.langbase().lstr(cfg.trainrounds as usize).to_string()
                    }
                }
                (2, 1) => {
                    if cfg.lct != 0 {
                        view.langbase().lstr(180).to_string()
                    } else {
                        view.langbase().lstr(185).to_string()
                    }
                }
                (2, 2) => {
                    if cfg.diff != 0 {
                        view.langbase().lstr(181).to_string()
                    } else {
                        view.langbase().lstr(186).to_string()
                    }
                }
                (2, 3) => {
                    if cfg.diffwc != 0 {
                        view.langbase().lstr(181).to_string()
                    } else {
                        view.langbase().lstr(186).to_string()
                    }
                }
                (2, 4) => {
                    if cfg.compactlist != 0 {
                        view.langbase().lstr(182).to_string()
                    } else {
                        view.langbase().lstr(187).to_string()
                    }
                }
                (2, 5) => {
                    if cfg.invback != 0 {
                        view.langbase().lstr(183).to_string()
                    } else {
                        view.langbase().lstr(188).to_string()
                    }
                }
                (2, 6) => {
                    if cfg.automatichrr != 0 {
                        view.langbase().lstr(182).to_string()
                    } else {
                        view.langbase().lstr(185).to_string()
                    }
                }
                (2, 7) => {
                    if cfg.goals != 0 {
                        view.langbase().lstr(180).to_string()
                    } else {
                        view.langbase().lstr(186).to_string()
                    }
                }
                (2, 8) => {
                    if cfg.seecomps >= 235 {
                        view.langbase().lstr(cfg.seecomps as usize).to_string()
                    } else {
                        format!("#{}", cfg.seecomps)
                    }
                }
                (2, 9) => wind_place_name(view.langbase(), cfg.windplace as usize),
                (2, 10) => {
                    if cfg.kosystem != 0 {
                        view.langbase().lstr(182).to_string()
                    } else {
                        view.langbase().lstr(185).to_string()
                    }
                }
                (3, 0) => {
                    if cfg.comphrs != 0 {
                        view.langbase().lstr(183).to_string()
                    } else {
                        view.langbase().lstr(187).to_string()
                    }
                }
                (3, 1) => {
                    if cfg.nosamename != 0 {
                        view.langbase().lstr(185).to_string()
                    } else {
                        view.langbase().lstr(180).to_string()
                    }
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };
        setup_item(view, cx, temp, entries, &value_str);
    }

    if view.modal.get().is_none() {
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
    let xx = 25;
    let yy = if index == 0 {
        (entries as i32) * 10 + 50
    } else {
        (index as i32) * 10 + 30
    };

    let row_label = format!("{}.", hex_char(index));
    cx.right_text((xx, yy), FONT_GOLD, row_label);

    let label_id = match (view.screen.get(), index) {
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

    cx.text((35, yy), FONT_BODY, view.langbase().lstr(label_id));

    if !value_str.is_empty() {
        cx.text((255, yy), FONT_GOLD, value_str);
    }
}
