use crate::components::modal::{confirmation_choice, ConfirmationChoice};
use crate::components::page_nav::cycle_index;
use crate::data::records::RecordStore;
use crate::jump::wind::WIND_POSITION_COUNT;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::store::GameState;
use crate::ui::{EventCx, Key, MenuAction, UiEvent};
use crate::views::jump::input::JumpKeyBindings;

use super::state::{setup_page, SetupItem, SetupModal};
use super::view::SetupView;

pub(crate) fn handle_event(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
) -> Option<RouteTarget> {
    if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) && view.modal.is_some() {
        view.modal = None;
        return None;
    }

    if matches!(event, UiEvent::KeyDown(Key::Escape)) && view.modal.is_none() {
        return Some(RouteTarget::MainMenu);
    }

    match view.modal {
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
            view.modal = Some(SetupModal::HillGoals(selected.saturating_sub(1)));
        }
        UiEvent::KeyDown(Key::Down) => {
            view.modal = Some(SetupModal::HillGoals((selected + 1).min(hill_count)));
        }
        UiEvent::KeyDown(Key::Home) => view.modal = Some(SetupModal::HillGoals(0)),
        UiEvent::KeyDown(Key::End) => {
            save_records(view, state);
            view.modal = None;
        }
        UiEvent::KeyDown(Key::Enter) if selected >= hill_count => {
            save_records(view, state);
            view.modal = None;
        }
        UiEvent::KeyDown(Key::Left) | UiEvent::Text('-') => {
            adjust_hill_goal(view, state, selected, -0.5)
        }
        UiEvent::KeyDown(Key::Right) | UiEvent::Text('+') => {
            adjust_hill_goal(view, state, selected, 0.5)
        }
        _ => {}
    }
    None
}

fn adjust_hill_goal(view: &SetupView, state: &mut GameState, selected: usize, delta: f64) {
    let hill = view.resources.hills.hill(selected);
    let Some(hill) = hill else {
        return;
    };
    let key = &hill.record_key;
    let current = state.records.hill_goal(key).copied().unwrap_or(0.0);
    let value = (current + delta).clamp(0.0, 250.0);
    state
        .records
        .set_hill_goal(key, (value * 10.0).round() / 10.0);
}

fn save_records(view: &SetupView, state: &GameState) {
    view.persistence().save_records(&state.records);
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

fn is_duplicate_key(config: &Config, item: usize, code: i32) -> bool {
    (0..5).any(|idx| idx != item && config_key(config, idx) == code)
}

fn set_config_key_if_unique(config: &mut Config, item: usize, code: i32) -> bool {
    if is_duplicate_key(config, item, code) {
        return false;
    }
    set_config_key(config, item, code);
    true
}

fn handle_configure_keys(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    selected: usize,
    capture: Option<usize>,
) -> Option<RouteTarget> {
    if let Some(item) = capture {
        if let Some(code) = JumpKeyBindings::code_for(event) {
            if set_config_key_if_unique(&mut state.config, item, code) {
                view.persistence().save_config(&state.config);
                view.modal = Some(SetupModal::ConfigureKeys {
                    selected,
                    capture: None,
                });
            }
        }
        return None;
    }

    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            view.modal = Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, -1),
                capture: None,
            });
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            view.modal = Some(SetupModal::ConfigureKeys {
                selected: cycle_index(selected, 7, 1),
                capture: None,
            });
        }
        UiEvent::KeyDown(Key::Home) => {
            view.modal = Some(SetupModal::ConfigureKeys {
                selected: 0,
                capture: None,
            })
        }
        UiEvent::KeyDown(Key::End) => {
            view.modal = Some(SetupModal::ConfigureKeys {
                selected: 6,
                capture: None,
            })
        }
        UiEvent::KeyDown(Key::Tab) => view.modal = None,
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => match selected {
            0..=4 => {
                view.modal = Some(SetupModal::ConfigureKeys {
                    selected,
                    capture: Some(selected),
                })
            }
            5 => {
                let defaults = Config::default();
                state.config.key_up = defaults.key_up;
                state.config.key_right = defaults.key_right;
                state.config.key_left = defaults.key_left;
                state.config.key_telemark = defaults.key_telemark;
                state.config.key_replay = defaults.key_replay;
                view.persistence().save_config(&state.config);
            }
            6 => view.modal = None,
            _ => {}
        },
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                match d as usize {
                    1..=5 => {
                        view.modal = Some(SetupModal::ConfigureKeys {
                            selected: d as usize - 1,
                            capture: None,
                        })
                    }
                    6 => {
                        view.modal = Some(SetupModal::ConfigureKeys {
                            selected: 5,
                            capture: None,
                        })
                    }
                    0 => {
                        view.modal = Some(SetupModal::ConfigureKeys {
                            selected: 6,
                            capture: None,
                        })
                    }
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
    if let UiEvent::Text(c) = event {
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
            view.persistence().save_config(&state.config);
            view.modal = None;
        }
    }
    None
}

fn handle_wind_place(
    view: &mut SetupView,
    state: &mut GameState,
    event: UiEvent,
    pos: usize,
) -> Option<RouteTarget> {
    let winds = usize::from(WIND_POSITION_COUNT);
    let items = winds + 1;
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            let new_pos = if pos == 0 { items - 1 } else { pos - 1 };
            view.modal = Some(SetupModal::WindPlace(new_pos));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_pos = if pos >= items - 1 { 0 } else { pos + 1 };
            view.modal = Some(SetupModal::WindPlace(new_pos));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            if pos < winds {
                state.config.wind_position = pos as i32;
                view.persistence().save_config(&state.config);
            }
            view.modal = None;
        }
        UiEvent::KeyDown(Key::Home) => view.modal = Some(SetupModal::WindPlace(0)),
        UiEvent::KeyDown(Key::End) => view.modal = Some(SetupModal::WindPlace(items - 1)),
        UiEvent::Text(c) if c.is_ascii_digit() => {
            let index = if c == '0' {
                items - 1
            } else {
                (c as u8 - b'1') as usize
            };
            if index < items {
                view.modal = Some(SetupModal::WindPlace(index));
            }
        }
        UiEvent::Text(c @ ('A' | 'a' | 'B' | 'b')) => {
            let index = c.to_ascii_uppercase() as usize - 'A' as usize + 9;
            view.modal = Some(SetupModal::WindPlace(index.min(items - 1)));
        }
        _ => {}
    }
    None
}

pub(crate) fn seecomp_options(view: &SetupView, state: &GameState) -> Vec<(usize, String)> {
    let lang = &view.resources.langbase;
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
        opts.push((v, lang.tr(v).to_string()));
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
            view.modal = Some(SetupModal::SeeComps(new_idx));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_idx = if idx >= opts.len() - 1 { 0 } else { idx + 1 };
            view.modal = Some(SetupModal::SeeComps(new_idx));
        }
        UiEvent::Text('-') => {
            let new_idx = if idx == 0 { opts.len() - 1 } else { idx - 1 };
            view.modal = Some(SetupModal::SeeComps(new_idx));
        }
        UiEvent::Text('+') => {
            let new_idx = if idx >= opts.len() - 1 { 0 } else { idx + 1 };
            view.modal = Some(SetupModal::SeeComps(new_idx));
        }
        UiEvent::KeyDown(Key::Home) => view.modal = Some(SetupModal::SeeComps(0)),
        UiEvent::KeyDown(Key::End) => view.modal = Some(SetupModal::SeeComps(opts.len() - 1)),
        UiEvent::KeyDown(Key::Enter) => {
            let cfg_val = opts[idx].0;
            state.config.visible_computers = cfg_val as i32;
            view.persistence().save_config(&state.config);
            view.modal = None;
        }
        UiEvent::KeyDown(Key::Delete) => {
            view.modal = None;
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
        UiEvent::Text(c)
            if confirmation_choice(c, &view.resources.langbase)
                == Some(ConfirmationChoice::Yes) =>
        {
            if let Some(SetupModal::ConfirmReset(kind)) = view.modal {
                let files = &view.resources.files;
                let Some(records) = (if kind == 1 {
                    RecordStore::bundled_default(files).ok()
                } else {
                    RecordStore::cleared_default(files).ok()
                }) else {
                    view.modal = None;
                    return None;
                };
                state.records = records.clone();
                view.persistence().save_records(&records);
            }
            view.modal = None;
        }
        UiEvent::KeyDown(_) | UiEvent::Text(_) => view.modal = None,
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
    let lang_count = view.resources.langbase.language_count();
    match event {
        UiEvent::KeyDown(Key::Up | Key::Left) => {
            let new_sel = cycle_index(sel, lang_count, -1);
            view.modal = Some(SetupModal::LanguagePicker(new_sel));
        }
        UiEvent::KeyDown(Key::Down | Key::Right) => {
            let new_sel = cycle_index(sel, lang_count, 1);
            view.modal = Some(SetupModal::LanguagePicker(new_sel));
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            view.resources.langbase.select(sel);
            state.config.language = view.resources.langbase.saved_language();
            view.persistence().save_config(&state.config);
            view.modal = None;
        }
        UiEvent::Text(c) if c.is_ascii_digit() => {
            if let Some(d) = c.to_digit(10) {
                let idx = d as usize;
                if idx >= 1 && idx <= lang_count {
                    view.modal = Some(SetupModal::LanguagePicker(idx - 1));
                }
            }
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
    let screen = view.screen;

    if matches!(event, UiEvent::KeyDown(Key::Tab | Key::F10)) {
        if screen == 0 {
            return Some(RouteTarget::MainMenu);
        }
        view.switch_screen(0);
        return None;
    }

    let mut ecx = EventCx::default();
    let msg = view.menu.event_action(&mut ecx, event);

    if ecx.is_consumed() && screen < view.selected_by_screen.len() {
        view.selected_by_screen[screen] = view.menu.selected();
    }

    match msg {
        Some(MenuAction::Item(action))
            if setup_page(screen).is_some_and(|page| action == page.items.len()) =>
        {
            if screen == 0 {
                return Some(RouteTarget::MainMenu);
            }
            view.switch_screen(0);
        }
        Some(MenuAction::Item(action)) => {
            if let Some(item) = setup_page(screen)
                .and_then(|page| page.items.get(action))
                .copied()
            {
                return activate_item(view, state, item);
            }
        }
        None => {}
    }

    None
}

fn activate_item(
    view: &mut SetupView,
    state: &mut GameState,
    item: SetupItem,
) -> Option<RouteTarget> {
    match item {
        SetupItem::Screen(screen) => view.switch_screen(screen),
        SetupItem::ConfigureKeys => {
            view.modal = Some(SetupModal::ConfigureKeys {
                selected: 0,
                capture: None,
            })
        }
        SetupItem::HillGoals => view.modal = Some(SetupModal::HillGoals(0)),
        SetupItem::HillMaker => return Some(RouteTarget::HillMakerSetup),
        SetupItem::Language => {
            let idx = view.resources.langbase.selected();
            view.modal = Some(SetupModal::LanguagePicker(idx));
        }
        SetupItem::SoundEffects => {
            state.config.sound_effects = i32::from(state.config.sound_effects == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::GraphicsDetail => {
            state.config.graphics_detail = i32::from(state.config.graphics_detail == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::NameSet => view.modal = Some(SetupModal::NameSetInput),
        SetupItem::TrainingRounds => {
            state.config.training_rounds = (state.config.training_rounds + 1) % 4;
            view.persistence().save_config(&state.config);
        }
        SetupItem::ExtraStatistics => {
            state.config.extra_statistics = i32::from(state.config.extra_statistics == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::EventGap => {
            state.config.event_gap = i32::from(state.config.event_gap == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::WorldCupGap => {
            state.config.wc_gap = i32::from(state.config.wc_gap == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::CompactResults => {
            state.config.compact_results = i32::from(state.config.compact_results == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::InvisibleBackground => {
            state.config.invisible_back = i32::from(state.config.invisible_back == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::AutoRecordReplay => {
            state.config.auto_hill_record_replay =
                i32::from(state.config.auto_hill_record_replay == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::Goals => {
            state.config.goals_enabled = i32::from(state.config.goals_enabled == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::VisibleComputers => {
            let current = state.config.visible_computers;
            let opts = seecomp_options(view, state);
            let idx = opts
                .iter()
                .position(|(v, _)| *v == current as usize)
                .unwrap_or(0);
            view.modal = Some(SetupModal::SeeComps(idx));
        }
        SetupItem::WindPosition => {
            let pos = state.config.wind_position;
            view.modal = Some(SetupModal::WindPlace(pos as usize));
        }
        SetupItem::KoSystem => {
            state.config.ko_system = i32::from(state.config.ko_system == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::ComputerHillRecords => {
            state.config.computer_hill_records = i32::from(state.config.computer_hill_records == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::UniqueComputerNames => {
            state.config.unique_computer_names = i32::from(state.config.unique_computer_names == 0);
            view.persistence().save_config(&state.config);
        }
        SetupItem::ResetBundledRecords => view.modal = Some(SetupModal::ConfirmReset(1)),
        SetupItem::ClearRecords => view.modal = Some(SetupModal::ConfirmReset(0)),
        SetupItem::ResetConfig => {
            state.config = Config::default();
            view.persistence().save_config(&state.config);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_jump_keys_cannot_duplicate_another_action() {
        let mut config = Config::default();
        let original = config.key_up;

        assert!(!set_config_key_if_unique(
            &mut config,
            0,
            Config::default().key_right
        ));
        assert_eq!(config.key_up, original);

        assert!(set_config_key_if_unique(&mut config, 0, original));
        assert_eq!(config.key_up, original);
        assert!(set_config_key_if_unique(&mut config, 0, 59));
        assert_eq!(config.key_up, 59);
    }

    #[test]
    fn duplicate_check_covers_every_configured_action() {
        let config = Config::default();
        let codes = (0..5)
            .map(|item| config_key(&config, item))
            .collect::<Vec<_>>();

        for (item, _) in codes.iter().enumerate().take(5) {
            assert!(!is_duplicate_key(&config, item, codes[item]));
            for (other, _) in codes.iter().enumerate().take(5) {
                if item != other {
                    assert!(is_duplicate_key(&config, item, codes[other]));
                }
            }
        }
    }
}
