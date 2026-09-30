use crate::competition::core::competitor::{
    active_profiles, computer_names_without_replacements, Competitor,
};
use crate::competition::koth::types::{KothParticipant, KothRuntime};
use crate::data::profile::ProfileStore;
use crate::rng::Random;
use crate::save::config::Config;

pub fn build_koth(
    config: &Config,
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    random_hill_count: usize,
    no_same_name: bool,
    mut rng: Random,
) -> KothRuntime {
    let koth_opponent_ids: Vec<usize> = config
        .koth_opponent_ids
        .iter()
        .map(|&v| v as usize)
        .collect();

    let jump_rounds_per_elimination = config.koth_rounds.clamp(1, 2) as u8;
    let packed_hill = if config.koth_hill >= 0 {
        Some(config.koth_hill as usize)
    } else {
        None
    };

    let hill_idx = packed_hill
        .filter(|&h| h < hill_count)
        .unwrap_or_else(|| rng.random_i32(random_hill_count as i32).max(0) as usize);

    let active = active_profiles(profiles);
    let filtered_names = if no_same_name {
        computer_names_without_replacements(computer_names, &active)
    } else {
        computer_names.to_vec()
    };

    let mut participants: Vec<KothParticipant> = Vec::new();
    let mut human_indices: Vec<usize> = Vec::new();

    for (order, &npc_id) in koth_opponent_ids.iter().enumerate() {
        let name = filtered_names
            .get(npc_id)
            .cloned()
            .unwrap_or_else(|| format!("Computer {}", npc_id + 1));
        let competitor = Competitor::computer(order, npc_id, name, None);
        participants.push(new_participant(competitor));
    }

    for (profile_idx, p) in &active {
        let idx = participants.len();
        let competitor = Competitor::from_profile(idx, *profile_idx, p, None);
        participants.push(new_participant(competitor));
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

pub fn apply_koth_pack(config: &mut Config, pack: u8) {
    let (count, pel, wind, rounds, maki) = match pack {
        1 => (20, (0..20).collect::<Vec<i32>>(), 0, 2, -1),
        2 => (14, (0..14).map(|i| i * 3 + 1).collect(), 1, 1, -1),
        3 => (10, (0..10).map(|i| i * 4 + 2).collect(), 0, 2, -1),
        4 => (8, (0..8).map(|i| i * 5 + 4).collect(), 1, 1, -1),
        5 => (7, (0..7).map(|i| i * 6 + 12).collect(), 0, 2, -1),
        6 => (6, (0..6).map(|i| i * 7 + 14).collect(), 1, 1, -1),
        _ => return,
    };
    config.koth_pack = i32::from(pack);
    config.koth_opponent_count = count;
    config.koth_opponent_ids = pel;
    config.koth_wind = wind;
    config.koth_rounds = rounds;
    config.koth_hill = maki;
}

fn new_participant(competitor: Competitor) -> KothParticipant {
    KothParticipant {
        competitor,
        total_points: 0.0,
        eliminated_in_round: u8::MAX,
        jumps: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefined_pack_zero_hill_uses_random_hill() {
        let config = Config {
            koth_pack: 1,
            koth_hill: -1,
            koth_opponent_ids: Vec::new(),
            ..Config::default()
        };

        let runtime = build_koth(
            &config,
            &ProfileStore::default(),
            &[],
            20,
            20,
            false,
            Random::new(0),
        );

        assert_eq!(runtime.hill_idx, 10);
    }

    #[test]
    fn zero_based_hill_selects_first_hill() {
        let config = Config {
            koth_pack: 0,
            koth_hill: 0,
            koth_opponent_ids: Vec::new(),
            ..Config::default()
        };

        let runtime = build_koth(
            &config,
            &ProfileStore::default(),
            &[],
            20,
            20,
            false,
            Random::new(0),
        );

        assert_eq!(runtime.hill_idx, 0);
    }

    #[test]
    fn selected_extra_hill_is_accepted_but_random_hills_stay_original() {
        let mut config = Config {
            koth_hill: 24,
            ..Config::default()
        };

        let selected = build_koth(
            &config,
            &ProfileStore::default(),
            &[],
            25,
            20,
            false,
            Random::new(0),
        );
        assert_eq!(selected.hill_idx, 24);

        config.koth_hill = -1;
        for seed in 0..100 {
            let random = build_koth(
                &config,
                &ProfileStore::default(),
                &[],
                25,
                20,
                false,
                Random::new(seed),
            );
            assert!(random.hill_idx < 20);
        }
    }

    #[test]
    fn opponent_ids_are_zero_based() {
        let config = Config {
            koth_pack: 0,
            koth_opponent_ids: vec![0, 2],
            ..Config::default()
        };
        let names = vec!["ONE".into(), "TWO".into(), "THREE".into()];

        let runtime = build_koth(
            &config,
            &ProfileStore::default(),
            &names,
            1,
            1,
            false,
            Random::new(0),
        );

        assert_eq!(runtime.participants[0].competitor.name, "ONE");
        assert_eq!(runtime.participants[0].competitor.ai_id, 0);
        assert_eq!(runtime.participants[1].competitor.name, "THREE");
        assert_eq!(runtime.participants[1].competitor.ai_id, 2);
    }
}
