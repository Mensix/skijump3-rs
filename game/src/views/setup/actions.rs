use crate::components::page_nav::cycle_index;
use crate::data::records::RecordStore;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::store::GameState;
use crate::views::jump::input::JumpKeyBindings;
use engine::oxide::input::{Key, UiEvent};
use engine::oxide::widgets::menu::PixelMenu;

use super::state::SetupModal;
use super::view::SetupView;

pub(crate) fn handle_event(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
) -> Option<RouteTarget> {
    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => handle_wind_place(view, state, event, pos),
        Some(SetupModal::SeeComps(val)) => handle_see_comps(view, state, event, val),
        Some(SetupModal::ConfirmReset(_)) => handle_confirm_reset(view, state, event),
        Some(SetupModal::LanguagePicker(sel)) => handle_language_picker(view, state, event, sel),
        Some(SetupModal::ConfigureKeys { selected, capture }) => {
            handle_configure_keys(view, state, event, selected, capture)
        }
        Some(SetupModal::NameSetInput) => handle_name_set_input(view, state, event),
        Some(SetupModal::HillGoals(selected)) => handle_hill_goals(view, state, event, selected),
        None => handle_screen_event(view, state, event),
    }
}

fn handle_hill_goals(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    selected: usize,
) -> Option<RouteTarget> {
    let hill_count = view.resources.hills.len().min(20);
    match event {
        UiEvent::KeyDown(Key::Up) => {
            view.modal
                .set(Some(SetupModal::HillGoals(selected.saturating_sub(1))));
        }
        UiEvent::KeyDown(Key::Down) => {
            view.modal
                .set(Some(SetupModal::HillGoals((selected + 1).min(hill_count))));
        }
        UiEvent::KeyDown(Key::Home) => view.modal.set(Some(SetupModal::HillGoals(0))),
        UiEvent::KeyDown(Key::End | Key::Escape) => {
            save_records(view, state);
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Enter) if selected >= hill_count => {
            save_records(view, state);
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Left) | UiEvent::Text('-') => {
            adjust_hill_goal(state, selected, -0.5, hill_count)
        }
        UiEvent::KeyDown(Key::Right) | UiEvent::Text('+') => {
            adjust_hill_goal(state, selected, 0.5, hill_count)
        }
        _ => {}
    }
    None
}

fn adjust_hill_goal(state: &mut GameState, selected: usize, delta: f64, hill_count: usize) {
    if selected >= hill_count {
        return;
    }
    if state.records.hill_goals.len() < hill_count {
        state.records.hill_goals.resize(hill_count, 0.0);
    }
    let value = (state.records.hill_goals[selected] + delta).clamp(0.0, 250.0);
    state.records.hill_goals[selected] = (value * 10.0).round() / 10.0;
}

fn save_records(view: &SetupView, state: &GameState) {
    if let Err(e) = view.save_manager().save_records(&state.records) {
        eprintln!("Warning: failed to save records: {e}");
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
    state: &mut GameState,
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
                    let duplicate =
                        (0..5).any(|idx| idx != item && config_key(&state.config, idx) == code);
                    if !duplicate {
                        set_config_key(&mut state.config, item, code);
                        if let Err(e) = view.save_manager().save_config(&state.config) {
                            eprintln!("Warning: failed to save config: {e}");
                        }
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
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            view.modal.set(Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, -1),
                capture: None,
            }));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            view.modal.set(Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, 1),
                capture: None,
            }));
        }
        UiEvent::KeyDown(Key::Home) => view.modal.set(Some(SetupModal::ConfigureKeys {
            selected: 0,
            capture: None,
        })),
        UiEvent::KeyDown(Key::End) => view.modal.set(Some(SetupModal::ConfigureKeys {
            selected: 6,
            capture: None,
        })),
        UiEvent::KeyDown(Key::Escape | Key::Tab) => view.modal.set(None),
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => match selected {
            0..=4 => view.modal.set(Some(SetupModal::ConfigureKeys {
                selected,
                capture: Some(selected),
            })),
            5 => {
                let defaults = Config::default();
                state.config.key_up = defaults.key_up;
                state.config.key_right = defaults.key_right;
                state.config.key_left = defaults.key_left;
                state.config.key_telemark = defaults.key_telemark;
                state.config.key_replay = defaults.key_replay;
                if let Err(e) = view.save_manager().save_config(&state.config) {
                    eprintln!("Warning: failed to save config: {e}");
                }
            }
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

fn handle_name_set_input(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
) -> Option<RouteTarget> {
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
                state.config.name_set_index = idx as i32;
                if let Err(e) = view.save_manager().save_config(&state.config) {
                    eprintln!("Warning: failed to save config: {e}");
                }
                view.modal.set(None);
            }
        }
        UiEvent::KeyDown(Key::Escape) => view.modal.set(None),
        _ => {}
    }
    None
}

fn handle_wind_place(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    pos: usize,
) -> Option<RouteTarget> {
    let winds = 11;
    let items = winds + 1; // 11 places + 0. exit
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            let new_pos = if pos == 0 { items - 1 } else { pos - 1 };
            view.modal.set(Some(SetupModal::WindPlace(new_pos)));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_pos = if pos >= items - 1 { 0 } else { pos + 1 };
            view.modal.set(Some(SetupModal::WindPlace(new_pos)));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            if pos < winds {
                let place = if pos < 8 { pos + 1 } else { pos + 3 };
                state.config.wind_position = place as i32;
                if let Err(e) = view.save_manager().save_config(&state.config) {
                    eprintln!("Warning: failed to save config: {e}");
                }
            }
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape) => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

pub(crate) fn seecomp_options(view: &SetupView, state: &GameState) -> Vec<(usize, String)> {
    let names = view
        .resources
        .namesets
        .names_for_config(state.config.name_set_index);
    let cats = [235, 236, 237, 238, 239, 240];
    let mut opts = Vec::new();
    for (i, name) in names.iter().enumerate() {
        if name != "Trainee" {
            opts.push((i, format!("#{} {}", i + 1, name)));
        }
    }
    for &v in &cats {
        opts.push((v, view.langbase().lstr(v).to_string()));
    }
    opts
}

fn handle_see_comps(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    idx: usize,
) -> Option<RouteTarget> {
    let opts = seecomp_options(view, state);
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            let new_idx = if idx == 0 { opts.len() - 1 } else { idx - 1 };
            view.modal.set(Some(SetupModal::SeeComps(new_idx)));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_idx = if idx >= opts.len() - 1 { 0 } else { idx + 1 };
            view.modal.set(Some(SetupModal::SeeComps(new_idx)));
        }
        UiEvent::KeyDown(Key::Enter) => {
            let cfg_val = opts[idx].0;
            state.config.visible_computers = cfg_val as i32;
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
            view.modal.set(None);
        }
        UiEvent::KeyDown(Key::Escape | Key::Delete) => {
            view.modal.set(None);
        }
        _ => {}
    }
    None
}

fn handle_confirm_reset(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
) -> Option<RouteTarget> {
    match event {
        UiEvent::Text(c) if c == 'y' || c == 'Y' => {
            if let Some(SetupModal::ConfirmReset(kind)) = view.modal.get() {
                let records = if kind == 1 {
                    RecordStore::bundled_default()
                } else {
                    RecordStore::cleared_default()
                };
                state.records = records.clone();
                if let Err(e) = view.save_manager().save_records(&records) {
                    eprintln!("Warning: failed to save records: {e}");
                }
            }
            view.modal.set(None);
        }
        UiEvent::KeyDown(_) | UiEvent::Text(_) => view.modal.set(None),
        _ => {}
    }
    None
}

fn handle_language_picker(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    sel: usize,
) -> Option<RouteTarget> {
    let langs = &view.langbase().languages;
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            let new_sel = cycle_index(sel, langs.len(), -1);
            view.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_sel = cycle_index(sel, langs.len(), 1);
            view.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            state.config.language = sel as i32;
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
            view.resources.langbase.selected.set(sel);
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

fn handle_screen_event(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
) -> Option<RouteTarget> {
    let screen = view.screen.get();
    let entries = view.menu.item_count();

    match event {
        UiEvent::KeyDown(Key::Up) => {
            let sel = view.menu.selected();
            let selected = cycle_index(sel, entries, -1);
            view.menu.set_selected(selected);
            if screen < view.selected_by_screen.len() {
                view.selected_by_screen[screen].set(selected);
            }
        }
        UiEvent::KeyDown(Key::Down) => {
            let sel = view.menu.selected();
            let selected = cycle_index(sel, entries, 1);
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
        UiEvent::KeyDown(Key::Home) => {
            view.menu.set_selected(0);
            if screen < view.selected_by_screen.len() {
                view.selected_by_screen[screen].set(0);
            }
        }
        UiEvent::KeyDown(Key::End) => {
            view.menu.set_selected(entries);
            if screen < view.selected_by_screen.len() {
                view.selected_by_screen[screen].set(entries);
            }
        }
        UiEvent::KeyDown(Key::Tab) => {
            if screen == 0 {
                return Some(RouteTarget::MainMenu);
            }
            view.switch_screen(0);
        }
        UiEvent::KeyDown(key) if PixelMenu::function_key_index(key).is_some() => {
            let n = PixelMenu::function_key_index(key).unwrap();
            if n == 10 || n > entries {
                if screen == 0 {
                    return Some(RouteTarget::MainMenu);
                }
                view.switch_screen(0);
            } else {
                let selected = n - 1;
                view.menu.set_selected(selected);
                if screen < view.selected_by_screen.len() {
                    view.selected_by_screen[screen].set(selected);
                }
                if let Some(route) = activate_item(view, state, screen, selected) {
                    return Some(route);
                }
            }
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
            if screen == 0 && sel == 4 {
                view.modal.set(Some(SetupModal::HillGoals(0)));
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
                if let Some(route) = activate_item(view, state, screen, sel) {
                    return Some(route);
                }
            }
        }
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                let n = d as usize;
                if n < entries {
                    view.menu.set_selected(n);
                    if screen < view.selected_by_screen.len() {
                        view.selected_by_screen[screen].set(n);
                    }
                } else if n == 0 {
                    view.menu.set_selected(entries);
                    if screen < view.selected_by_screen.len() {
                        view.selected_by_screen[screen].set(entries);
                    }
                }
            }
        }
        UiEvent::Text(c) if matches!(c, 'A'..='L' | 'a'..='l') => {
            let n = c.to_ascii_uppercase() as usize - 'A' as usize + 10;
            if n <= entries {
                view.menu.set_selected(n);
                if screen < view.selected_by_screen.len() {
                    view.selected_by_screen[screen].set(n);
                }
            }
        }
        _ => {}
    }
    None
}

fn activate_item(
    view: &mut SetupView,
    state: &mut GameState,
    screen: usize,
    item: usize,
) -> Option<RouteTarget> {
    match (screen, item) {
        (0, 0..=2) => view.switch_screen(item),
        (0, 3) => view.modal.set(Some(SetupModal::ConfigureKeys {
            selected: 0,
            capture: None,
        })),
        (0, 4) => view.modal.set(Some(SetupModal::HillGoals(0))),
        (0, 5) => return Some(RouteTarget::HillMakerSetup),
        (1, 0) => {
            let current = state.config.language;
            let idx = if current >= 0 { current as usize } else { 0 };
            let langs = &view.langbase().languages;
            let idx = idx.min(langs.len().saturating_sub(1));
            view.modal.set(Some(SetupModal::LanguagePicker(idx)));
        }
        (1, 1) => {
            state.config.sound_effects = i32::from(state.config.sound_effects == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (1, 2) => {
            state.config.graphics_detail = i32::from(state.config.graphics_detail == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (1, 3) => view.modal.set(Some(SetupModal::NameSetInput)),
        (2, 0) => {
            state.config.training_rounds = (state.config.training_rounds + 1) % 4;
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 1) => {
            state.config.extra_statistics = i32::from(state.config.extra_statistics == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 2) => {
            state.config.event_gap = i32::from(state.config.event_gap == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 3) => {
            state.config.wc_gap = i32::from(state.config.wc_gap == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 4) => {
            state.config.compact_results = i32::from(state.config.compact_results == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 5) => {
            state.config.invisible_back = i32::from(state.config.invisible_back == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 6) => {
            state.config.auto_hill_record_replay =
                i32::from(state.config.auto_hill_record_replay == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 7) => {
            state.config.goals_enabled = i32::from(state.config.goals_enabled == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (2, 8) => {
            let current = state.config.visible_computers;
            let opts = seecomp_options(view, state);
            let idx = opts
                .iter()
                .position(|(v, _)| *v == current as usize)
                .unwrap_or(0);
            view.modal.set(Some(SetupModal::SeeComps(idx)));
        }
        (2, 9) => {
            let place = state.config.wind_position;
            let pos = if place <= 8 { place - 1 } else { place - 3 };
            view.modal.set(Some(SetupModal::WindPlace(pos as usize)));
        }
        (2, 10) => {
            state.config.ko_system = i32::from(state.config.ko_system == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (3, 0) => {
            state.config.computer_hill_records = i32::from(state.config.computer_hill_records == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (3, 1) => {
            state.config.unique_computer_names = i32::from(state.config.unique_computer_names == 0);
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        (3, 2) => view.modal.set(Some(SetupModal::ConfirmReset(1))),
        (3, 3) => view.modal.set(Some(SetupModal::ConfirmReset(0))),
        (3, 4) => {
            state.config = Config::default();
            if let Err(e) = view.save_manager().save_config(&state.config) {
                eprintln!("Warning: failed to save config: {e}");
            }
        }
        _ => {}
    }
    None
}
