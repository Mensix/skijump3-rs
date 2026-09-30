use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::jump::wind::WIND_POSITION_COUNT;
use crate::store::GameState;
use crate::text::input::key_name;
use crate::ui::UiCanvas;

use super::state::{hex_char, setup_page, wind_place_name, SetupItem, SetupModal};
use super::view::SetupView;

pub(crate) fn paint_content(view: &SetupView, state: &GameState, cx: &mut dyn UiCanvas) {
    let lang = &view.resources.langbase;
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

            let winds = usize::from(WIND_POSITION_COUNT);
            for apu in 0..winds {
                let yy = ((apu + 1) as i32) * 10 + 34;
                let name = wind_place_name(&view.resources.langbase, apu);
                cx.right_text((85, yy), FONT_GOLD, &format!("{}.", apu + 1));
                cx.text((90, yy), FONT_BODY, &name);
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
            cx.fill((69, 79, 183, 53), FILL_PURPLE);
            cx.pattern_fill((70, 80, 181, 51), BG_PURPLE);
            cx.text((80, 90), FONT_GOLD, &format!("{} {}", label, lang.tr(192)));
            cx.text((80, 110), FONT_GOLD, lang.tr(193));
        }
        Some(SetupModal::LanguagePicker(sel)) => {
            cx.fill((74, 41, 172, 145), FILL_PURPLE);
            cx.fill((75, 42, 170, 143), BG_PURPLE);
            cx.text((100, 50), FONT_BODY, "PLEASE CHOOSE A LANGUAGE:");
            for (i, lang) in view.resources.langbase.languages().iter().enumerate() {
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

fn render_hill_goals(view: &SetupView, state: &GameState, cx: &mut dyn UiCanvas, selected: usize) {
    let lang = &view.resources.langbase;
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
            cx.right_text((18, y), FONT_GOLD, &format!("{}.", idx + 1));
            cx.text((24, y), FONT_BODY, &hill.name);
            let goal = state
                .records
                .hill_goal(&hill.record_key)
                .copied()
                .unwrap_or(0.0);
            cx.right_text((200, y), goal_color, &format_distance(goal));
            cx.right_text((250, y), FONT_TEAL, &format_distance(hill.kr as f64));
            let record = state
                .records
                .hill_record(&hill.record_key)
                .map_or(0.0, |r| r.len);
            cx.right_text((300, y), FONT_TEAL, &format_distance(record));
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
    cx: &mut dyn UiCanvas,
    selected: usize,
    capture: Option<usize>,
) {
    let lang = &view.resources.langbase;
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
        cx.right_text((x, y), FONT_GOLD, &format!("{}.", temp + 1));
        cx.text((x + 10, y), FONT_BODY, lang.tr(temp + 331));
        if temp < 5 {
            let key_text = if capture == Some(item) {
                "".to_string()
            } else {
                key_name(keys[item], &view.resources.langbase)
            };
            if capture == Some(item) {
                cx.fill((180, y - 2, 140, 10), FILL_GRAY);
                cx.pattern_fill((180, y - 2, 140, 10), BG_PURPLE);
                cx.fill((183, y - 2, 9, 11), FILL_GRAY);
                if view.cursor_blink.visible(11, 10) {
                    cx.fill((185, y + 6, 5, 1), FONT_BODY);
                }
            }
            cx.text((x + 160, y), FONT_GOLD, &key_text);
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

fn render_screen(view: &SetupView, state: &GameState, cx: &mut dyn UiCanvas) {
    let lang = &view.resources.langbase;
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);

    cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

    let Some(page) = setup_page(view.screen) else {
        return;
    };
    cx.text((30, 6), FONT_BODY, lang.tr(page.title_id));

    let entries = page.items.len();
    setup_item(view, cx, 0, entries, page.trailing_label_id, "");
    for (index, item) in page.items.iter().copied().enumerate() {
        let value = setup_value(view, state, item);
        if item == SetupItem::NameSet {
            let hint = view
                .resources
                .namesets
                .title_for_config(state.config.name_set_index);
            cx.text((40, 78), FONT_GRAY, &format!("*** {hint} ***"));
        }
        setup_item(view, cx, index + 1, entries, item.label_id(), &value);
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

fn setup_value(view: &SetupView, state: &GameState, item: SetupItem) -> String {
    let lang = &view.resources.langbase;
    let choice =
        |enabled: bool, yes: usize, no: usize| lang.tr(if enabled { yes } else { no }).to_string();
    match item {
        SetupItem::Language => {
            let languages = lang.languages();
            usize::try_from(state.config.language)
                .ok()
                .and_then(|index| languages.get(index))
                .map_or_else(|| "Undecided".to_string(), |info| info.name.clone())
        }
        SetupItem::SoundEffects => choice(state.config.sound_effects != 0, 6, 7),
        SetupItem::GraphicsDetail => choice(state.config.graphics_detail == 0, 13, 14),
        SetupItem::NameSet => {
            let index = state.config.name_set_index;
            index.to_string()
        }
        SetupItem::TrainingRounds => {
            let rounds = state.config.training_rounds as usize;
            lang.tr(if rounds == 0 { 9 } else { rounds }).to_string()
        }
        SetupItem::ExtraStatistics => choice(state.config.extra_statistics != 0, 180, 185),
        SetupItem::EventGap => choice(state.config.event_gap != 0, 181, 186),
        SetupItem::WorldCupGap => choice(state.config.wc_gap != 0, 181, 186),
        SetupItem::CompactResults => choice(state.config.compact_results != 0, 182, 187),
        SetupItem::InvisibleBackground => choice(state.config.invisible_back != 0, 183, 188),
        SetupItem::AutoRecordReplay => choice(state.config.auto_hill_record_replay != 0, 182, 185),
        SetupItem::Goals => choice(state.config.goals_enabled != 0, 180, 186),
        SetupItem::VisibleComputers if state.config.visible_computers >= 235 => {
            lang.tr(state.config.visible_computers as usize).to_string()
        }
        SetupItem::VisibleComputers => format!("#{}", state.config.visible_computers + 1),
        SetupItem::WindPosition => wind_place_name(lang, state.config.wind_position as usize),
        SetupItem::KoSystem => choice(state.config.ko_system != 0, 182, 185),
        SetupItem::ComputerHillRecords => choice(state.config.computer_hill_records != 0, 183, 187),
        SetupItem::UniqueComputerNames => choice(state.config.unique_computer_names != 0, 185, 180),
        SetupItem::Screen(_)
        | SetupItem::ConfigureKeys
        | SetupItem::HillGoals
        | SetupItem::HillMaker
        | SetupItem::ResetBundledRecords
        | SetupItem::ClearRecords
        | SetupItem::ResetConfig => String::new(),
    }
}

fn setup_item(
    view: &SetupView,
    cx: &mut dyn UiCanvas,
    index: usize,
    entries: usize,
    label_id: usize,
    value_str: &str,
) {
    let lang = &view.resources.langbase;
    let xx = 25;
    let yy = if index == 0 {
        (entries as i32) * 10 + 50
    } else {
        (index as i32) * 10 + 30
    };

    let row_label = format!("{}.", hex_char(index));
    cx.right_text((xx, yy), FONT_GOLD, &row_label);

    cx.text((35, yy), FONT_BODY, lang.tr(label_id));

    if !value_str.is_empty() {
        cx.text((255, yy), FONT_GOLD, value_str);
    }
}
