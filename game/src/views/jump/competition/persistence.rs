use crate::store::{GameStateRef, ResourcesRef};
use std::cell::Cell;

pub(crate) fn save_profiles_and_records_once(
    saved: &Cell<bool>,
    resources: &ResourcesRef,
    state: &GameStateRef,
) {
    if saved.replace(true) {
        return;
    }
    save_profiles(resources, state);
    save_records(resources, state);
}

fn save_profiles(resources: &ResourcesRef, state: &GameStateRef) {
    if let Err(e) = resources
        .save_manager
        .save_players(&state.borrow().profiles)
    {
        eprintln!("Warning: failed to save players: {e}");
    }
}

fn save_records(resources: &ResourcesRef, state: &GameStateRef) {
    if let Ok(s) = state.try_borrow() {
        if let Err(e) = resources.save_manager.save_records(&s.records) {
            eprintln!("Warning: failed to save records: {e}");
        }
    }
}
