use crate::competition::core::competitor::{
    active_profiles, computer_names_without_replacements, Competitor,
};
use crate::competition::koth::types::{KothParticipant, KothRuntime};
use crate::data::profile::ProfileStore;
use crate::rng::Random;
use crate::save::config::Config;
use crate::save::SaveRef;
use crate::store::GameState;

#[must_use]
pub fn build_koth(
    config: &Config,
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    no_same_name: bool,
    mut rng: Random,
) -> KothRuntime {
    let koth_opponent_ids: Vec<usize> = config
        .koth_opponent_ids
        .iter()
        .map(|&v| v as usize)
        .collect();

    let jump_rounds_per_elimination = config.koth_rounds.clamp(1, 2) as u8;
    let _pack = config.koth_pack.clamp(1, 6) as u8;
    let packed_hill = if config.koth_hill >= 0 {
        Some(config.koth_hill as usize)
    } else {
        None
    };

    // resolve hill: use packed_hill if set, otherwise random
    let hill_idx = packed_hill
        .filter(|&h| h < hill_count)
        .unwrap_or_else(|| rng.random_i32(hill_count as i32).max(0) as usize);

    // build participants: NPCs from koth_opponent_ids, then humans from profiles
    let active = active_profiles(profiles);
    let filtered_names = if no_same_name {
        computer_names_without_replacements(computer_names, &active)
    } else {
        computer_names.to_vec()
    };

    let mut participants: Vec<KothParticipant> = Vec::new();
    let mut human_indices: Vec<usize> = Vec::new();

    // NPCs from koth_opponent_ids
    for (order, &npc_id) in koth_opponent_ids.iter().enumerate() {
        let name = filtered_names
            .get(npc_id % filtered_names.len().max(1))
            .cloned()
            .unwrap_or_else(|| format!("Computer {}", npc_id + 1));
        let competitor = Competitor::computer(order, npc_id, name, None);
        participants.push(KothParticipant {
            competitor,
            total_points: 0.0,
            eliminated_in_round: u8::MAX,
            jumps: Vec::new(),
        });
    }

    // human profiles
    for (profile_idx, p) in &active {
        let idx = participants.len();
        let competitor = Competitor::from_profile(idx, *profile_idx, p, None);
        participants.push(KothParticipant {
            competitor,
            total_points: 0.0,
            eliminated_in_round: u8::MAX,
            jumps: Vec::new(),
        });
        human_indices.push(idx);
    }

    KothRuntime::new(
        participants,
        human_indices,
        hill_idx,
        jump_rounds_per_elimination,
        rng,
    )
}

/// Pascal `getkoth` — fills config fields based on pack number (1..6, 0=custom no-op).
pub fn apply_koth_pack(state: &mut GameState, save_manager: &SaveRef, pack: u8) {
    let (count, pel, wind, rounds, maki) = match pack {
        1 => (20, (1..=20).collect::<Vec<i32>>(), 0, 2, 0),
        2 => (14, (1..=14).map(|i| i * 3 - 1).collect(), 1, 1, 0),
        3 => (10, (1..=10).map(|i| i * 4 - 1).collect(), 0, 2, 0),
        4 => (8, (1..=8).map(|i| i * 5 + 1).collect(), 1, 1, 0),
        5 => (7, (1..=7).map(|i| i * 6 + 13).collect(), 0, 2, 0),
        6 => (6, (1..=6).map(|i| i * 7 + 15).collect(), 1, 1, 0),
        _ => return, // custom pack, leave as-is
    };
    state.config.koth_opponent_count = count;
    state.config.koth_opponent_ids = pel;
    state.config.koth_wind = wind;
    state.config.koth_rounds = rounds;
    state.config.koth_hill = maki;
    let _ = save_manager.save_config(&state.config);
}
