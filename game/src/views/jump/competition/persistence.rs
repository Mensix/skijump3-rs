use crate::save::SaveRef;
use crate::store::GameStateRef;
use std::cell::Cell;

pub(crate) fn save_profiles_and_records_once(
    saved: &Cell<bool>,
    save_manager: &SaveRef,
    state: &GameStateRef,
) {
    if saved.replace(true) {
        return;
    }
    save_profiles(save_manager, state);
    save_records(save_manager, state);
}

fn save_profiles(save_manager: &SaveRef, state: &GameStateRef) {
    if let Err(e) = save_manager.save_players(&state.borrow().profiles) {
        eprintln!("Warning: failed to save players: {e}");
    }
}

fn save_records(save_manager: &SaveRef, state: &GameStateRef) {
    if let Ok(s) = state.try_borrow() {
        if let Err(e) = save_manager.save_records(&s.records) {
            eprintln!("Warning: failed to save records: {e}");
        }
    }
}
