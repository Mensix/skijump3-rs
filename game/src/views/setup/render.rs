use crate::components::screen;
use crate::gfx::palette::{BG_LEFT, FILL_BORDER, FILL_DIM, FONT_DEFAULT, FONT_HEADER, FONT_HELP};
use engine::ui::Element;

use super::state::{hex_char, wind_place_name, SetupModal};
use super::view::SetupView;

pub(crate) fn elements(view: &SetupView) -> Vec<Element> {
    let mut els = Vec::new();
    render_screen(view, &mut els);

    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => {
            els.extend(rect_bg(54, 19, 222, 162));
            els.push(Element::text(
                view.langbase().lstr(221),
                75,
                30,
                FONT_HEADER,
                false,
            ));

            let winds = 11;
            for apu1 in 1..=winds {
                let yy = (apu1 as i32) * 10 + 34;
                let name = if apu1 <= 8 {
                    wind_place_name(view.langbase(), apu1)
                } else {
                    wind_place_name(view.langbase(), apu1 + 2)
                };
                els.push(Element::text(format!("{apu1}."), 85, yy, FONT_HEADER, true));
                let color = if (apu1 - 1) == pos {
                    FONT_HEADER
                } else {
                    FONT_DEFAULT
                };
                els.push(Element::text(name, 90, yy, color, false));
            }

            let yy = (winds * 10 + 34 + 20) as i32;
            els.push(Element::text(
                format!("0.{}", view.langbase().lstr(154)),
                85,
                yy,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::text(
                view.langbase().lstr(150),
                75,
                175,
                FONT_HELP,
                false,
            ));
        }
        Some(SetupModal::SeeComps(val)) => {
            els.extend(rect_bg(74, 79, 172, 54));
            els.push(Element::text(
                view.langbase().lstr(220),
                85,
                85,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::text(
                view.langbase().lstr(150),
                85,
                95,
                FONT_HELP,
                false,
            ));
            let display = if val > 240 {
                view.langbase().lstr(val).to_string()
            } else {
                format!("#{val}")
            };
            els.push(Element::fillbox(85, 105, 150, 20, FILL_DIM));
            els.push(Element::text(display, 95, 112, FONT_HEADER, false));
        }
        Some(SetupModal::ConfirmReset(kind)) => {
            els.extend(rect_bg(69, 79, 182, 52));
            let label = if kind == 1 {
                view.langbase().lstr(190)
            } else {
                view.langbase().lstr(191)
            };
            els.push(Element::text(
                format!("{} {}", label, view.langbase().lstr(192)),
                80,
                90,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::text(
                view.langbase().lstr(193),
                80,
                110,
                FONT_DEFAULT,
                false,
            ));
        }
        Some(SetupModal::LanguagePicker(sel)) => {
            let langs = &view.langbase().languages;
            els.push(Element::fillbox(74, 41, 173, 146, FILL_BORDER));
            els.push(Element::fillbox(75, 42, 171, 144, BG_LEFT));
            els.push(Element::text(
                "PLEASE CHOOSE A LANGUAGE:",
                100,
                50,
                FONT_DEFAULT,
                false,
            ));
            for (i, name) in langs.iter().enumerate() {
                let yy = ((i + 1) as i32) * 8 + 55;
                els.push(Element::center_text(name, 155, yy, FONT_HEADER));
            }
            let bx = 112 - 6;
            let by = 64 - 3 + (sel as i32) * 8;
            els.push(Element::box_(bx, by, 100 + 1, 8 + 1, FONT_DEFAULT));
        }
        None => {}
    }

    els
}

fn rect_bg(x: i32, y: i32, w: i32, h: i32) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, w, h, FILL_BORDER),
        Element::fillbox(x + 1, y + 1, w - 2, h - 2, BG_LEFT),
    ]
}

fn render_screen(view: &SetupView, els: &mut Vec<Element>) {
    els.extend(screen::new_screen(1));

    let title_id = match view.screen.get() {
        0 => 175,
        1 => 176,
        2 => 177,
        3 => 178,
        _ => return,
    };
    els.push(Element::text(
        view.langbase().lstr(title_id),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));

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
                    els.push(Element::text(hint.to_string(), 40, 78, FONT_HELP, false));
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
        setup_item(view, els, temp, entries, &value_str);
    }

    let sel = view.menu.selected();
    if sel <= entries {
        let by = if sel < entries {
            40 - 3 + (sel as i32) * 10
        } else {
            (entries as i32) * 10 + 50 - 3
        };
        els.push(Element::box_(35 - 6, by, 221 + 1, 10 + 1, FONT_DEFAULT));
    }
}

fn setup_item(
    view: &SetupView,
    els: &mut Vec<Element>,
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
    els.push(Element::text(row_label, xx, yy, FONT_HEADER, true));

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

    els.push(Element::text(
        view.langbase().lstr(label_id),
        35,
        yy,
        FONT_DEFAULT,
        false,
    ));

    if !value_str.is_empty() {
        els.push(Element::text(value_str, 255, yy, FONT_HEADER, false));
    }
}
