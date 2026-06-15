use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_LEFT, BLACK, FILL_BORDER, FILL_DIM, FONT_DEFAULT, FONT_HEADER,
    FONT_HELP,
};
use engine::oxide::PaintCx;

use super::state::{hex_char, key_name, wind_place_name, SetupModal};
use super::view::SetupView;

pub(crate) fn paint_content(view: &SetupView, cx: &mut PaintCx<'_>) {
    if let Some(SetupModal::ConfigureKeys { selected, capture }) = view.modal.get() {
        render_configure_keys(view, cx, selected, capture);
        return;
    }

    render_screen(view, cx);

    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => {
            cx.fill((54, 19, 222, 162), FILL_BORDER);
            cx.fill((55, 20, 220, 160), BG_LEFT);
            cx.text((75, 30), FONT_HEADER, view.langbase().lstr(221));

            let winds = 11;
            for apu1 in 1..=winds {
                let yy = (apu1 as i32) * 10 + 34;
                let name = if apu1 <= 8 {
                    wind_place_name(view.langbase(), apu1)
                } else {
                    wind_place_name(view.langbase(), apu1 + 2)
                };
                cx.right_text((85, yy), FONT_HEADER, format!("{apu1}."));
                cx.text((90, yy), FONT_DEFAULT, name);
            }

            let yy = (winds * 10 + 34 + 20) as i32;
            cx.right_text((85, yy), FONT_HEADER, "0.");
            cx.text((90, yy), FONT_DEFAULT, view.langbase().lstr(154));
            cx.stroke(
                (70 - 6, 44 - 3 + (pos as i32) * 10, 140 + 1, 10 + 1),
                FONT_DEFAULT,
            );
        }
        Some(SetupModal::SeeComps(val)) => {
            cx.fill((74, 79, 172, 54), FILL_BORDER);
            cx.fill((75, 80, 170, 52), BG_LEFT);
            cx.text((85, 85), FONT_DEFAULT, view.langbase().lstr(220));
            cx.text((85, 95), FONT_HELP, view.langbase().lstr(150));
            let display = if val > 240 {
                view.langbase().lstr(val).to_string()
            } else {
                format!("#{val}")
            };
            cx.fill((85, 105, 150, 20), FILL_DIM);
            cx.text((95, 112), FONT_HEADER, display);
        }
        Some(SetupModal::ConfirmReset(kind)) => {
            cx.fill((69, 79, 182, 52), FILL_BORDER);
            cx.fill((70, 80, 180, 50), BG_LEFT);
            let label = if kind == 1 {
                view.langbase().lstr(190)
            } else {
                view.langbase().lstr(191)
            };
            cx.text(
                (80, 90),
                FONT_DEFAULT,
                format!("{} {}", label, view.langbase().lstr(192)),
            );
            cx.text((80, 110), FONT_DEFAULT, view.langbase().lstr(193));
        }
        Some(SetupModal::LanguagePicker(sel)) => {
            let langs = &view.langbase().languages;
            cx.fill((74, 41, 172, 145), FILL_BORDER);
            cx.fill((75, 42, 170, 143), BG_LEFT);
            cx.text((100, 50), FONT_DEFAULT, "PLEASE CHOOSE A LANGUAGE:");
            for (i, name) in langs.iter().enumerate() {
                let yy = ((i + 1) as i32) * 8 + 55;
                cx.center_text((155, yy), FONT_HEADER, name);
            }
            cx.stroke((112 - 6, 64 - 3 + (sel as i32) * 8, 100 + 1, 8 + 1), FONT_DEFAULT);
        }
        Some(SetupModal::ConfigureKeys { .. }) => {}
        Some(SetupModal::NameSetInput) => {
            let yy = 70;
            cx.fill((248, yy - 2, 71, 10), FILL_DIM);
            cx.stroke((248, yy - 2, 71, 10), FONT_DEFAULT);
        }
        None => {}
    }
}

fn render_configure_keys(
    view: &SetupView,
    cx: &mut PaintCx<'_>,
    selected: usize,
    capture: Option<usize>,
) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_DIM);
    cx.pattern_fill((0, 20, 320, 180), BG_LEFT);
    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
    cx.text((30, 6), FONT_DEFAULT, view.langbase().lstr(199));

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
        cx.right_text((x, y), FONT_HEADER, format!("{temp}."));
        cx.text((x + 10, y), FONT_DEFAULT, view.langbase().lstr(temp + 330));
        if temp < 6 {
            let key_text = if capture == Some(item) {
                "".to_string()
            } else {
                key_name(keys[item], view.langbase())
            };
            if capture == Some(item) {
                cx.fill((180, y - 2, 139, 9), BG_LEFT);
                cx.pattern_fill((180, y - 2, 139, 9), FILL_DIM);
            }
            cx.text((x + 160, y), FONT_HEADER, key_text);
        }
    }

    y += 20;
    cx.right_text((x, y), FONT_HEADER, "0.");
    cx.text((x + 10, y), FONT_DEFAULT, view.langbase().lstr(337));

    let by = if selected < 6 {
        40 - 3 + (selected as i32) * 10
    } else {
        y - 3
    };
    cx.stroke((35 - 6, by, 150 + 1, 10 + 1), FONT_DEFAULT);
}

fn render_screen(view: &SetupView, cx: &mut PaintCx<'_>) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_DIM);
    cx.pattern_fill((0, 20, 320, 180), BG_LEFT);

    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let title_id = match view.screen.get() {
        0 => 175,
        1 => 176,
        2 => 177,
        3 => 178,
        _ => return,
    };
    cx.text((30, 6), FONT_DEFAULT, view.langbase().lstr(title_id));

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
                    cx.text((40, 78), FONT_HELP, hint);
                    format!("{n}")
                }
                (2, 0) => {
                    if cfg.trainrounds == 0 {
                        view.langbase().lstr(9).to_string()
                    } else {
                        format!("{}", cfg.trainrounds)
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
                    if cfg.seecomps > 240 {
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

    let sel = view.menu.selected();
    if sel <= entries {
        let by = if sel < entries {
            40 - 3 + (sel as i32) * 10
        } else {
            (entries as i32) * 10 + 50 - 3
        };
        cx.stroke((35 - 6, by, 221 + 1, 10 + 1), FONT_DEFAULT);
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
    cx.right_text((xx, yy), FONT_HEADER, row_label);

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

    cx.text((35, yy), FONT_DEFAULT, view.langbase().lstr(label_id));

    if !value_str.is_empty() {
        cx.text((255, yy), FONT_HEADER, value_str);
    }
}
