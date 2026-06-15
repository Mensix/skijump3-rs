use crate::components::page_nav::cycle_index;
use crate::data::records::RecordStore;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::views::jump::input::JumpKeyBindings;
use engine::oxide::input::{Key, UiEvent};

use super::state::SetupModal;
use super::view::SetupView;

pub(crate) fn handle_event(view: &mut SetupView, event: UiEvent) -> Option<RouteTarget> {
    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => handle_wind_place(view, event, pos),
        Some(SetupModal::SeeComps(val)) => handle_see_comps(view, event, val),
        Some(SetupModal::ConfirmReset(_)) => handle_confirm_reset(view, event),
        Some(SetupModal::LanguagePicker(sel)) => handle_language_picker(view, event, sel),
        Some(SetupModal::ConfigureKeys { selected, capture }) => {
            handle_configure_keys(view, event, selected, capture)
        }
        Some(SetupModal::NameSetInput) => handle_name_set_input(view, event),
        None => handle_screen_event(view, event),
    }
}

fn config_key(config: &Config, item: usize) -> i32 {
    match item {
        0 => config.key_up,
        1 => config.key_right,
        2 => config.key_left,
        3 => config.key_telemark,
        4 => config.key_replay,
        _ => 0,
    }
}

fn set_config_key(config: &mut Config, item: usize, code: i32) {
    match item {
        0 => config.key_up = code,
        1 => config.key_right = code,
        2 => config.key_left = code,
        3 => config.key_telemark = code,
        4 => config.key_replay = code,
        _ => {}
    }
}

fn handle_configure_keys(
    view: &mut SetupView,
    event: UiEvent,
    selected: usize,
    capture: Option<usize>,
) -> Option<RouteTarget> {
    if let Some(item) = capture {
        match event {
            UiEvent::KeyDown(Key::Escape) => {
                view.modal.set(Some(SetupModal::ConfigureKeys {
                    selected,
                    capture: None,
                }));
            }
            _ => {
                if let Some(code) = JumpKeyBindings::code_for(event) {
                    let duplicate = {
                        let cfg = view.config();
                        (0..5).any(|idx| idx != item && config_key(&cfg, idx) == code)
                    };
                    if !duplicate {
                        view.save_manager()
                            .update_config(|cfg| set_config_key(cfg, item, code));
                        view.modal.set(Some(SetupModal::ConfigureKeys {
                            selected,
                            capture: None,
                        }));
                    }
                }
            }
        }
        return None;
    }

    match event {
        UiEvent::KeyDown(Key::Up) => {
            view.modal.set(Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, -1),
                capture: None,
            }));
        }
        UiEvent::KeyDown(Key::Down) => {
            view.modal.set(Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, 1),
                capture: None,
            }));
        }
        UiEvent::KeyDown(Key::Escape) => view.modal.set(None),
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => match selected {
            0..=4 => view.modal.set(Some(SetupModal::ConfigureKeys {
                selected,
                capture: Some(selected),
            })),
            5 => view.save_manager().update_config(|cfg| {
                let defaults = Config::default();
                cfg.key_up = defaults.key_up;
                cfg.key_right = defaults.key_right;
                cfg.key_left = defaults.key_left;
                cfg.key_telemark = defaults.key_telemark;
                cfg.key_replay = defaults.key_replay;
            }),
            6 => view.modal.set(None),
            _ => {}
        },
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                match d as usize {
                    1..=5 => view.modal.set(Some(SetupModal::ConfigureKeys {
                        selected: d as usize - 1,
                        capture: None,
                    })),
                    6 => view.modal.set(Some(SetupModal::ConfigureKeys {
                        selected: 5,
                        capture: None,
                    })),
                    0 => view.modal.set(Some(SetupModal::ConfigureKeys {
                        selected: 6,
                        capture: None,
                    })),
                    _ => {}
                }
            }
        }
        _ => {}
    }
    None
}

fn handle_name_set_input(view: &mut SetupView, event: UiEvent) -> Option<RouteTarget> {
    match event {
        UiEvent::Text(c) => {
            let ns_len = view.resources.namesets.len();
            let idx = if c.is_ascii_digit() {
                (c as u8 - b'0') as usize
            } else if c.is_ascii_uppercase() {
                (c as u8 - b'A') as usize + 10
            } else if c.is_ascii_lowercase() {
                (c.to_ascii_uppercase() as u8 - b'A') as usize + 10
            } else {
                return None;
            };
            if idx < ns_len {
                view.save_manager()
                    .update_config(|cfg| cfg.namenumber = idx as i32);
            }
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape) => view.modal.set(None),
        _ => {}
    }
    None
}

fn handle_wind_place(view: &mut SetupView, event: UiEvent, pos: usize) -> Option<RouteTarget> {
    let winds = 11;
    match event {
        UiEvent::KeyDown(Key::Up) => {
            let new_pos = if pos == 0 { winds - 1 } else { pos - 1 };
            view.modal.set(Some(SetupModal::WindPlace(new_pos)));
        }
        UiEvent::KeyDown(Key::Down) => {
            let new_pos = if pos >= winds - 1 { 0 } else { pos + 1 };
            view.modal.set(Some(SetupModal::WindPlace(new_pos)));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            let place = if pos < 8 { pos + 1 } else { pos + 3 };
            view.save_manager()
                .update_config(|cfg| cfg.windplace = place as i32);
            view.store.set_wind_place(place as u8);
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape) => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

fn handle_see_comps(view: &mut SetupView, event: UiEvent, mut val: usize) -> Option<RouteTarget> {
    let num_players: i32 = 250;
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            if val > 1 {
                val -= 1
            } else {
                val = 240;
            }
            if val < 235 && val > num_players as usize {
                val = (num_players as usize).saturating_sub(11);
            }
            view.modal.set(Some(SetupModal::SeeComps(val)));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            if val >= 240 {
                val = 1;
            } else {
                val += 1;
            }
            if val > num_players as usize - 11 && val < 235 {
                val = 235;
            }
            view.modal.set(Some(SetupModal::SeeComps(val)));
        }
        UiEvent::KeyDown(Key::Enter) => {
            view.save_manager()
                .update_config(|cfg| cfg.seecomps = val as i32);
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape) => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

fn handle_confirm_reset(view: &mut SetupView, event: UiEvent) -> Option<RouteTarget> {
    match event {
        UiEvent::Text(c) if c == 'y' || c == 'Y' => {
            if let Some(SetupModal::ConfirmReset(kind)) = view.modal.get() {
                let records = if kind == 1 {
                    RecordStore::bundled_default()
                } else {
                    RecordStore::cleared_default()
                };
                view.store.replace_records(records.clone());
                if let Err(e) = view.save_manager().save_records(&records) {
                    eprintln!("Warning: failed to save records: {e}");
                }
            }
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape | Key::Enter) => {
            view.modal.set(None);
        }
        UiEvent::Text(c) if c == 'n' || c == 'N' => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

fn handle_language_picker(view: &mut SetupView, event: UiEvent, sel: usize) -> Option<RouteTarget> {
    let langs = &view.langbase().languages;
    match event {
        UiEvent::KeyDown(Key::Up) => {
            let new_sel = cycle_index(sel, langs.len(), -1);
            view.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
        }
        UiEvent::KeyDown(Key::Down) => {
            let new_sel = cycle_index(sel, langs.len(), 1);
            view.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            view.save_manager().set_language(sel);
            view.modal.set(None);
        }
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                let idx = d as usize;
                if idx >= 1 && idx <= langs.len() {
                    view.modal.set(Some(SetupModal::LanguagePicker(idx - 1)));
                }
            }
        }
        UiEvent::KeyDown(Key::Escape) => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

fn handle_screen_event(view: &mut SetupView, event: UiEvent) -> Option<RouteTarget> {
    let screen = view.screen.get();
    let entries = view.menu.item_count();

    match event {
        UiEvent::KeyDown(Key::Up) => {
            let sel = view.menu.selected();
            let selected = cycle_index(sel, entries + 1, -1);
            view.menu.set_selected(selected);
            if screen < view.selected_by_screen.len() {
                view.selected_by_screen[screen].set(selected);
            }
        }
        UiEvent::KeyDown(Key::Down) => {
            let sel = view.menu.selected();
            let selected = cycle_index(sel, entries + 1, 1);
            view.menu.set_selected(selected);
            if screen < view.selected_by_screen.len() {
                view.selected_by_screen[screen].set(selected);
            }
        }
        UiEvent::KeyDown(Key::Escape) => {
            if screen == 0 {
                return Some(RouteTarget::MainMenu);
            }
            view.switch_screen(0);
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            let sel = view.menu.selected();
            if screen == 0 && sel == 3 {
                view.modal.set(Some(SetupModal::ConfigureKeys {
                    selected: 0,
                    capture: None,
                }));
                return None;
            }
            if screen == 0 && sel == 5 {
                return Some(RouteTarget::HillMakerSetup);
            }
            if sel >= entries {
                if screen == 0 {
                    return Some(RouteTarget::MainMenu);
                }
                view.switch_screen(0);
            } else {
                activate_item(view, screen, sel);
            }
        }
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                let n = d as usize;
                if n >= 1 && n <= entries {
                    view.menu.set_selected(n - 1);
                    if screen < view.selected_by_screen.len() {
                        view.selected_by_screen[screen].set(n - 1);
                    }
                } else if n == 0 {
                    view.menu.set_selected(entries);
                    if screen < view.selected_by_screen.len() {
                        view.selected_by_screen[screen].set(entries);
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn activate_item(view: &mut SetupView, screen: usize, item: usize) {
    match (screen, item) {
        (0, 0..=2) => view.switch_screen(item + 1),
        (0, 3) => view.modal.set(Some(SetupModal::ConfigureKeys {
            selected: 0,
            capture: None,
        })),
        (1, 0) => {
            let current = view.config().languagenumber;
            let idx = if current >= 0 { current as usize } else { 0 };
            let langs = &view.langbase().languages;
            let idx = idx.min(langs.len().saturating_sub(1));
            view.modal.set(Some(SetupModal::LanguagePicker(idx)));
        }
        (1, 1) => view
            .save_manager()
            .update_config(|cfg| cfg.beeppi = i32::from(cfg.beeppi == 0)),
        (1, 2) => view
            .save_manager()
            .update_config(|cfg| cfg.gdetail = i32::from(cfg.gdetail == 0)),
        (1, 3) => view.modal.set(Some(SetupModal::NameSetInput)),
        (2, 0) => view
            .save_manager()
            .update_config(|cfg| cfg.trainrounds = (cfg.trainrounds + 1) % 4),
        (2, 1) => view
            .save_manager()
            .update_config(|cfg| cfg.lct = i32::from(cfg.lct == 0)),
        (2, 2) => view
            .save_manager()
            .update_config(|cfg| cfg.diff = i32::from(cfg.diff == 0)),
        (2, 3) => view
            .save_manager()
            .update_config(|cfg| cfg.diffwc = i32::from(cfg.diffwc == 0)),
        (2, 4) => view
            .save_manager()
            .update_config(|cfg| cfg.compactlist = i32::from(cfg.compactlist == 0)),
        (2, 5) => view
            .save_manager()
            .update_config(|cfg| cfg.invback = i32::from(cfg.invback == 0)),
        (2, 6) => view
            .save_manager()
            .update_config(|cfg| cfg.automatichrr = i32::from(cfg.automatichrr == 0)),
        (2, 7) => view
            .save_manager()
            .update_config(|cfg| cfg.goals = i32::from(cfg.goals == 0)),
        (2, 8) => {
            let current = view.config().seecomps;
            let idx = if current >= 1 { current as usize } else { 240 };
            view.modal.set(Some(SetupModal::SeeComps(idx)));
        }
        (2, 9) => {
            let place = view.config().windplace;
            let pos = if place <= 8 { place - 1 } else { place - 3 };
            view.modal.set(Some(SetupModal::WindPlace(pos as usize)));
        }
        (2, 10) => view
            .save_manager()
            .update_config(|cfg| cfg.kosystem = i32::from(cfg.kosystem == 0)),
        (3, 0) => view
            .save_manager()
            .update_config(|cfg| cfg.comphrs = i32::from(cfg.comphrs == 0)),
        (3, 1) => view
            .save_manager()
            .update_config(|cfg| cfg.nosamename = i32::from(cfg.nosamename == 0)),
        (3, 2) => view.modal.set(Some(SetupModal::ConfirmReset(1))),
        (3, 3) => view.modal.set(Some(SetupModal::ConfirmReset(0))),
        (3, 4) => {
            view.save_manager().update_config(|cfg| {
                *cfg = Config::default();
            });
            let cfg = view.config();
            view.store.set_wind_place(cfg.windplace as u8);
        }
        _ => {}
    }
}
