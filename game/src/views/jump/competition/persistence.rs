use crate::store::{ResourcesRef, StoreRef};
use std::cell::Cell;

pub(crate) fn save_profiles_and_records_once(
    saved: &Cell<bool>,
    resources: &ResourcesRef,
    store: &StoreRef,
) {
    if saved.replace(true) {
        return;
    }
    save_profiles(resources, store);
    save_records(resources, store);
}

fn save_profiles(resources: &ResourcesRef, store: &StoreRef) {
    let result = store.with_profiles(|profiles| resources.save_manager.save_players(profiles));
    if let Err(e) = result {
        eprintln!("Warning: failed to save players: {e}");
    }
}

fn save_records(resources: &ResourcesRef, store: &StoreRef) {
    let result = store.with_records(|records| resources.save_manager.save_records(records));
    if let Err(e) = result {
        eprintln!("Warning: failed to save records: {e}");
    }
}
