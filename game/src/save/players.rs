use serde::{Deserialize, Serialize};

use crate::data::profile::{ProfileStore, MAX_ACTIVE_PROFILES, MAX_PROFILES};
use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
struct ProfilesFile {
    #[serde(flatten)]
    store: ProfileStore,
}

impl ProfileStore {
    pub fn from_toml_bytes(data: &[u8]) -> Self {
        let file: ProfilesFile = parse_toml(data);

        let store = &file.store;

        assert!(!store.profiles.is_empty(), "players.toml has no profiles");
        assert!(
            store.profiles.len() <= MAX_PROFILES,
            "players.toml has {} profiles (max {MAX_PROFILES})",
            store.profiles.len()
        );
        assert!(
            store.active_order.len() <= MAX_ACTIVE_PROFILES,
            "players.toml has {} active profiles (max {MAX_ACTIVE_PROFILES})",
            store.active_order.len()
        );
        for (i, &idx) in store.active_order.iter().enumerate() {
            assert!(
                idx < store.profiles.len(),
                "active_order[{i}] = {idx} out of range (profiles: {n})",
                i = i,
                idx = idx,
                n = store.profiles.len()
            );
        }

        file.store
    }

    pub fn to_toml_bytes(&self) -> Vec<u8> {
        let file = ProfilesFile {
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_all_fields() {
        let mut store = ProfileStore::new();
        let p = &mut store.profiles[0];
        p.name = "TEST JUMPER".to_string();
        p.real_name = "Test".to_string();
        p.suit_color = [24, 28, 63];
        p.ski_color = [33, 60, 33];
        p.replace = 1;
        p.coach_style = 2;
        p.skip_qualification = 1;
        p.total_jumps = 100;
        p.world_cups = 5;
        p.legs_won = 3;
        p.world_cups_won = 1;
        p.best_result = "1 (1.)".to_string();
        p.best_4h_result = "3 (-)".to_string();
        p.best_wc_jump = 120.0;
        p.best_wc_hill_idx = 2;
        p.best_jump = 125.0;
        p.besthill_idx = 3;
        p.best_hill_file = "TESTHILL".to_string();
        p.best_points = 2500;
        p.best_4h_points = 2400.0;
        p.koth_level = 5;

        let bytes = store.to_toml_bytes();
        let parsed = ProfileStore::from_toml_bytes(&bytes);

        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.active_order, store.active_order);

        let pp = &parsed.profiles[0];
        assert_eq!(pp.name, "TEST JUMPER");
        assert_eq!(pp.real_name, "Test");
        assert_eq!(pp.suit_color, [24, 28, 63]);
        assert_eq!(pp.ski_color, [33, 60, 33]);
        assert_eq!(pp.replace, 1);
        assert_eq!(pp.coach_style, 2);
        assert_eq!(pp.skip_qualification, 1);
        assert_eq!(pp.total_jumps, 100);
        assert_eq!(pp.world_cups, 5);
        assert_eq!(pp.legs_won, 3);
        assert_eq!(pp.world_cups_won, 1);
        assert_eq!(pp.best_result, "1 (1.)");
        assert_eq!(pp.best_4h_result, "3 (-)");
        assert_eq!(pp.best_wc_jump, 120.0);
        assert_eq!(pp.best_wc_hill_idx, 2);
        assert_eq!(pp.best_jump, 125.0);
        assert_eq!(pp.besthill_idx, 3);
        assert_eq!(pp.best_hill_file, "TESTHILL");
        assert_eq!(pp.best_points, 2500);
        assert_eq!(pp.best_4h_points, 2400.0);
        assert_eq!(pp.koth_level, 5);
    }

    #[test]
    fn display_fields_are_not_persisted() {
        let mut store = ProfileStore::new();
        store.profiles[0].best_wc_hill_display = "SHOULD_NOT_SAVE".to_string();
        store.profiles[0].best_hill_display = "SHOULD_NOT_SAVE".to_string();

        let bytes = store.to_toml_bytes();
        let text = String::from_utf8(bytes).unwrap();
        assert!(
            !text.contains("SHOULD_NOT_SAVE"),
            "display fields leaked into TOML"
        );
    }
}
