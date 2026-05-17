use crate::jump::config::JumpConfig;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::{JumpParticipant, JumpPolicy, JumpRunner, JumpRunnerRenderEnv};
use crate::store::{ResourcesRef, StoreRef};

pub(crate) fn new_runner_with_env(
    hill_idx: usize,
    start_gate: i32,
    participant: JumpParticipant,
    policy: JumpPolicy,
    resources: &ResourcesRef,
    store: &StoreRef,
) -> JumpRunner {
    let hill = resources.hills.hill(hill_idx).cloned();
    let terrain = resources.hill_terrain(hill_idx).map(|t| (*t).clone());
    let mut snow = SnowSystem::new();

    if terrain.is_ok() && hill.is_some() {
        let mut rng = store.rng.borrow_mut();
        let mut wind = store.wind.borrow_mut();
        wind.initialize(&mut rng, store.wind_place.get());

        if store.first_event.get() {
            let snow_count = calculate_snow_count(&mut rng);
            snow.set_count(snow_count, &mut rng);
            wind.sample(&mut rng);
            store.first_event.set(false);
        }
    }

    let record_distance = store
        .records
        .borrow()
        .hill_record(hill_idx)
        .map_or(0, |r| r.len as i32);

    JumpRunner::new(JumpConfig {
        hill_idx,
        hill,
        terrain,
        snow,
        start_gate,
        participant,
        policy,
        record_distance,
        phase_label: String::new(),
    })
}

pub(crate) fn set_runner_hill(runner: &mut JumpRunner, hill_idx: usize, resources: &ResourcesRef) {
    let hill = resources.hills.hill(hill_idx).cloned();
    let terrain = resources.hill_terrain(hill_idx).map(|t| (*t).clone());
    runner.set_hill(hill_idx, hill, terrain);
}

pub(crate) fn runner_elements(
    runner: &mut JumpRunner,
    resources: &ResourcesRef,
    store: &StoreRef,
) -> Vec<engine::ui::Element> {
    let mut rng = store.rng.borrow_mut();
    let mut wind = store.wind.borrow_mut();
    let records = store.records.borrow();

    runner.elements(JumpRunnerRenderEnv {
        font: &resources.font,
        langbase: &resources.langbase,
        hills: &resources.hills,
        records: &records,
        rng: &mut rng,
        wind: &mut wind,
    })
}
