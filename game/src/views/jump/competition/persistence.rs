use crate::store::{ResourcesRef, StoreRef};
use std::cell::Cell;

pub(crate) fn save_profiles_once(saved: &Cell<bool>, resources: &ResourcesRef, store: &StoreRef) {
    if saved.replace(true) {
        return;
    }
    save_profiles(resources, store);
}

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
    if let Err(e) = resources.save_manager.save_players(&store.profiles()) {
        eprintln!("Warning: failed to save players: {e}");
    }
}

fn save_records(resources: &ResourcesRef, store: &StoreRef) {
    if let Some(records) = store.try_records() {
        if let Err(e) = resources.save_manager.save_records(&records) {
            eprintln!("Warning: failed to save records: {e}");
        }
    }
}
