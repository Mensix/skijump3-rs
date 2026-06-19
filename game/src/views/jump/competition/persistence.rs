use crate::save::SaveRef;
use crate::store::GameState;
use std::cell::Cell;

pub(crate) fn save_profiles_and_records_once(
    saved: &Cell<bool>,
    save_manager: &SaveRef,
    state: &GameState,
) {
    if saved.replace(true) {
        return;
    }
    save_profiles(save_manager, state);
    save_records(save_manager, state);
}

fn save_profiles(save_manager: &SaveRef, state: &GameState) {
    if let Err(e) = save_manager.save_players(&state.profiles) {
        eprintln!("Warning: failed to save players: {e}");
    }
}

fn save_records(save_manager: &SaveRef, state: &GameState) {
    if let Err(e) = save_manager.save_records(&state.records) {
        eprintln!("Warning: failed to save records: {e}");
    }
}
