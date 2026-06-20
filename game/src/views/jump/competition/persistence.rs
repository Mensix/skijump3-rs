use crate::save::SaveRef;
use crate::store::GameState;

pub(crate) fn save_profiles_and_records_once(
    saved: &mut bool,
    save_manager: &SaveRef,
    state: &GameState,
) {
    if *saved {
        return;
    }
    *saved = true;
    save_profiles(save_manager, state);
    save_records(save_manager, state);
}

fn save_profiles(save_manager: &SaveRef, state: &GameState) {
    save_manager.save_players(&state.profiles);
}

fn save_records(save_manager: &SaveRef, state: &GameState) {
    save_manager.save_records(&state.records);
}
