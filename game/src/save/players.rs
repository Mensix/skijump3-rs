use serde::{Deserialize, Serialize};

use crate::data::profile::{ProfileStore, MAX_ACTIVE_PROFILES, MAX_PROFILES};
use crate::save::SaveError;

/// TOML wrapper to version the file — mirrors save/config.rs pattern.
#[derive(Debug, Deserialize, Serialize)]
struct ProfilesFile {
    format_version: u32,
    #[serde(flatten)]
    store: ProfileStore,
}

impl ProfileStore {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, SaveError> {
        let text = std::str::from_utf8(data).map_err(SaveError::Utf8)?;
        let file: ProfilesFile =
            toml::from_str(text).map_err(|e| SaveError::Serialization(e.to_string()))?;
        if file.format_version != 1 {
            return Err(SaveError::Serialization(format!(
                "Unsupported players format_version: {}",
                file.format_version
            )));
        }

        let store = &file.store;

        if store.profiles.is_empty() {
            return Err(SaveError::Serialization(
                "players.toml has no profiles".to_string(),
            ));
        }
        if store.profiles.len() > MAX_PROFILES {
            return Err(SaveError::Serialization(format!(
                "players.toml has {} profiles (max {MAX_PROFILES})",
                store.profiles.len()
            )));
        }
        if store.active_order.len() > MAX_ACTIVE_PROFILES {
            return Err(SaveError::Serialization(format!(
                "players.toml has {} active profiles (max {MAX_ACTIVE_PROFILES})",
                store.active_order.len()
            )));
        }
        for (i, &idx) in store.active_order.iter().enumerate() {
            if idx >= store.profiles.len() {
                return Err(SaveError::Serialization(format!(
                    "active_order[{i}] = {idx} out of range (profiles: {n})",
                    i = i,
                    idx = idx,
                    n = store.profiles.len()
                )));
            }
        }

        Ok(file.store)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, SaveError> {
        let file = ProfilesFile {
            format_version: 1,
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|e| SaveError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid TOML for a single profile entry.
    fn one_profile_toml() -> Vec<u8> {
        r#"format_version = 1
active_order = [0]

[[profiles]]
name = "X"
real_name = ""
suit_color = 0
ski_color = 0
replace = 0
coach_style = 1
skip_quali = 0
total_jumps = 0
world_cups = 0
legs_won = 0
world_cups_won = 0
best_result = "-"
best_4h_result = "-"
best_wc_jump = 0
bestwchill = 0
best_jump = 0
besthill_idx = 0
besthillfile = ""
bestpoints = 0
best4points = 0
koth_level = 0
"#
        .as_bytes()
        .to_vec()
    }

    #[test]
    fn rejects_empty_profiles() {
        let bytes = b"format_version = 1\nactive_order = []\n";
        let result = ProfileStore::from_toml_bytes(bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no profiles"));
    }

    #[test]
    fn rejects_too_many_profiles() {
        let mut toml = one_profile_toml();
        for _ in 0..21 {
            toml.extend_from_slice(
                b"\n[[profiles]]\nname = \"P\"\nreal_name = \"\"\nsuit_color = 0\nski_color = 0\nreplace = 0\ncoach_style = 1\nskip_quali = 0\ntotal_jumps = 0\nworld_cups = 0\nlegs_won = 0\nworld_cups_won = 0\nbest_result = \"-\"\nbest_4h_result = \"-\"\nbest_wc_jump = 0\nbestwchill = 0\nbest_jump = 0\nbesthill_idx = 0\nbesthillfile = \"\"\nbestpoints = 0\nbest4points = 0\nkoth_level = 0\n",
            );
        }
        let result = ProfileStore::from_toml_bytes(&toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("20"));
    }

    #[test]
    fn rejects_out_of_range_active_order() {
        let text = String::from_utf8(one_profile_toml()).unwrap();
        let text = text.replace("active_order = [0]", "active_order = [99]");
        let result = ProfileStore::from_toml_bytes(text.as_bytes());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of range"));
    }

    #[test]
    fn roundtrip_preserves_all_fields() {
        let mut store = ProfileStore::new();
        let p = &mut store.profiles[0];
        p.name = "TEST JUMPER".to_string();
        p.real_name = "Test".to_string();
        p.suit_color = 3;
        p.ski_color = 2;
        p.replace = 1;
        p.coach_style = 2;
        p.skip_quali = 1;
        p.total_jumps = 100;
        p.world_cups = 5;
        p.legs_won = 3;
        p.world_cups_won = 1;
        p.best_result = "1 (1.)".to_string();
        p.best_4h_result = "3 (-)".to_string();
        p.best_wc_jump = 1200;
        p.bestwchill = 2;
        p.best_jump = 1250;
        p.besthill_idx = 3;
        p.besthillfile = "TESTHILL".to_string();
        p.bestpoints = 2500;
        p.best4points = 2400;
        p.koth_level = 5;

        let bytes = store.to_toml_bytes().unwrap();
        let parsed = ProfileStore::from_toml_bytes(&bytes).unwrap();

        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.active_order, store.active_order);

        let pp = &parsed.profiles[0];
        assert_eq!(pp.name, "TEST JUMPER");
        assert_eq!(pp.real_name, "Test");
        assert_eq!(pp.suit_color, 3);
        assert_eq!(pp.ski_color, 2);
        assert_eq!(pp.replace, 1);
        assert_eq!(pp.coach_style, 2);
        assert_eq!(pp.skip_quali, 1);
        assert_eq!(pp.total_jumps, 100);
        assert_eq!(pp.world_cups, 5);
        assert_eq!(pp.legs_won, 3);
        assert_eq!(pp.world_cups_won, 1);
        assert_eq!(pp.best_result, "1 (1.)");
        assert_eq!(pp.best_4h_result, "3 (-)");
        assert_eq!(pp.best_wc_jump, 1200);
        assert_eq!(pp.bestwchill, 2);
        assert_eq!(pp.best_jump, 1250);
        assert_eq!(pp.besthill_idx, 3);
        assert_eq!(pp.besthillfile, "TESTHILL");
        assert_eq!(pp.bestpoints, 2500);
        assert_eq!(pp.best4points, 2400);
        assert_eq!(pp.koth_level, 5);
    }

    #[test]
    fn display_fields_are_not_persisted() {
        let mut store = ProfileStore::new();
        store.profiles[0].best_wc_hill_display = "SHOULD_NOT_SAVE".to_string();
        store.profiles[0].best_hill_display = "SHOULD_NOT_SAVE".to_string();

        let bytes = store.to_toml_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(
            !text.contains("SHOULD_NOT_SAVE"),
            "display fields leaked into TOML"
        );
    }

    #[test]
    fn rejects_bad_format_version() {
        let text = String::from_utf8(one_profile_toml()).unwrap();
        let text = text.replace("format_version = 1", "format_version = 99");
        let result = ProfileStore::from_toml_bytes(text.as_bytes());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("format_version"));
    }

    #[test]
    fn rejects_invalid_toml() {
        let bytes = b"garbage [[[toml]]]\n";
        let result = ProfileStore::from_toml_bytes(bytes);
        assert!(result.is_err());
    }
}
