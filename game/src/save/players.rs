use serde::{Deserialize, Serialize};

use crate::data::profile::{ProfileStore, MAX_ACTIVE_PROFILES, MAX_PROFILES};
use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfilesFile {
    format_version: u32,
    #[serde(flatten)]
    store: ProfileStore,
}

const FORMAT_VERSION: u32 = 1;

impl ProfileStore {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, String> {
        let file: ProfilesFile = parse_toml(data)?;
        if file.format_version != FORMAT_VERSION {
            return Err(format!(
                "unsupported players format {}",
                file.format_version
            ));
        }

        let store = &file.store;

        if store.profiles.is_empty() || store.profiles.len() > MAX_PROFILES {
            return Err("invalid profile count".to_string());
        }
        if store.active_order.len() > MAX_ACTIVE_PROFILES {
            return Err("too many active profiles".to_string());
        }
        for (i, &idx) in store.active_order.iter().enumerate() {
            if idx >= store.profiles.len() {
                return Err(format!("active_order[{i}] is out of range"));
            }
        }
        let mut active = store.active_order.clone();
        active.sort_unstable();
        active.dedup();
        if active.len() != store.active_order.len() || store.active_order.is_empty() {
            return Err("active_order must contain unique profiles".to_string());
        }
        for (index, profile) in store.profiles.iter().enumerate() {
            if profile.name.trim().is_empty()
                || profile.name.len() > 80
                || profile.real_name.len() > 160
            {
                return Err(format!("profile {index} has an invalid name"));
            }
            if profile.coach_style > 10
                || profile.skip_qualification > 2
                || profile.koth_level > 100
            {
                return Err(format!("profile {index} setting is out of range"));
            }
            if profile
                .replace
                .is_some_and(|replacement| replacement > 10_000)
                || !profile.best_wc_jump.is_finite()
                || profile.best_wc_jump < 0.0
                || !profile.best_jump.is_finite()
                || profile.best_jump < 0.0
                || !profile.best_4h_points.is_finite()
            {
                return Err(format!("profile {index} statistic is invalid"));
            }
        }

        Ok(file.store)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, String> {
        let file = ProfilesFile {
            format_version: FORMAT_VERSION,
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::color::Rgb6;

    #[test]
    fn roundtrip_preserves_all_fields() {
        let mut store = ProfileStore::new();
        let p = &mut store.profiles[0];
        p.name = "TEST JUMPER".to_string();
        p.real_name = "Test".to_string();
        p.suit_color = Rgb6([24, 28, 63]);
        p.ski_color = Rgb6([33, 60, 33]);
        p.replace = Some(0);
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
        p.best_wc_hill_display = "World Cup Hill".to_string();
        p.best_hill_display = "Overall Hill".to_string();
        p.best_points = 2500;
        p.best_4h_points = 2400.0;
        p.koth_level = 5;

        let Ok(bytes) = store.to_toml_bytes() else {
            return;
        };
        let parsed = ProfileStore::from_toml_bytes(&bytes).unwrap();

        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.active_order, store.active_order);

        let pp = &parsed.profiles[0];
        assert_eq!(pp.name, "TEST JUMPER");
        assert_eq!(pp.real_name, "Test");
        assert_eq!(pp.suit_color, Rgb6([24, 28, 63]));
        assert_eq!(pp.ski_color, Rgb6([33, 60, 33]));
        assert_eq!(pp.replace, Some(0));
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
        assert_eq!(pp.best_wc_hill_display, "World Cup Hill");
        assert_eq!(pp.best_hill_display, "Overall Hill");
        assert_eq!(pp.best_points, 2500);
        assert_eq!(pp.best_4h_points, 2400.0);
        assert_eq!(pp.koth_level, 5);
    }

    #[test]
    fn display_fields_survive_restart_roundtrip() {
        let mut store = ProfileStore::new();
        store.profiles[0].best_wc_hill_display = "Vikersund".to_string();
        store.profiles[0].best_hill_display = "Custom Peak".to_string();

        let Ok(bytes) = store.to_toml_bytes() else {
            return;
        };
        let restarted = ProfileStore::from_toml_bytes(&bytes).unwrap();

        assert_eq!(restarted.profiles[0].best_wc_hill_display, "Vikersund");
        assert_eq!(restarted.profiles[0].best_hill_display, "Custom Peak");
    }
}
