use crate::components::page_nav::cycle_index;
use crate::route::RouteTarget;
use engine::oxide::input::{Key, UiEvent};

use super::state::SetupModal;
use super::view::SetupView;

pub(crate) fn handle_event(view: &mut SetupView, event: UiEvent) -> Option<RouteTarget> {
    match view.modal.get() {
        Some(SetupModal::WindPlace(pos)) => handle_wind_place(view, event, pos),
        Some(SetupModal::SeeComps(val)) => handle_see_comps(view, event, val),
        Some(SetupModal::ConfirmReset(kind)) => handle_confirm_reset(view, event, kind),
        Some(SetupModal::LanguagePicker(sel)) => handle_language_picker(view, event, sel),
        None => handle_screen_event(view, event),
    }
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

fn handle_confirm_reset(view: &mut SetupView, event: UiEvent, _kind: u8) -> Option<RouteTarget> {
    match event {
        UiEvent::Text(c) if c == 'y' || c == 'Y' => {
            let _ = _kind;
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
            view.menu.set_selected(cycle_index(sel, entries + 1, -1));
        }
        UiEvent::KeyDown(Key::Down) => {
            let sel = view.menu.selected();
            view.menu.set_selected(cycle_index(sel, entries + 1, 1));
        }
        UiEvent::KeyDown(Key::Escape) => {
            if screen == 0 {
                return Some(RouteTarget::MainMenu);
            }
            view.switch_screen(0);
        }
        UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
            let sel = view.menu.selected();
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
                } else if n == 0 {
                    view.menu.set_selected(entries);
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
        (1, 3) => {
            let ns_len = view.resources.namesets.len();
            view.save_manager()
                .update_config(|cfg| cfg.namenumber = (cfg.namenumber + 1) % ns_len as i32);
        }
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
        (3, 4) => view.save_manager().update_config(|cfg| {
            *cfg = crate::save::config::Config::default();
        }),
        _ => {}
    }
}
