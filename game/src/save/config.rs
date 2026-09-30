use serde::{Deserialize, Serialize};

use crate::jump::wind::WIND_POSITION_COUNT;
use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    format_version: u32,
    #[serde(flatten)]
    config: Config,
}

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub computer_hill_records: i32,
    pub extra_statistics: i32,
    pub event_gap: i32,
    pub compact_results: i32,
    pub invisible_back: i32,
    pub auto_hill_record_replay: i32,
    pub sound_effects: i32,
    pub unique_computer_names: i32,
    pub goals_enabled: i32,
    pub wc_gap: i32,
    pub ko_system: i32,
    pub language: i32,
    pub training_rounds: i32,
    pub name_set_index: i32,
    pub last_custom_cup_file: String,
    pub graphics_detail: i32,
    pub visible_computers: i32,
    pub koth_wind: i32,
    pub koth_rounds: i32,
    pub koth_pack: i32,
    pub koth_hill: i32,
    pub koth_opponent_count: i32,
    pub koth_opponent_ids: Vec<i32>,
    pub key_up: i32,
    pub key_right: i32,
    pub key_left: i32,
    pub key_telemark: i32,
    pub key_replay: i32,
    pub wind_position: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            computer_hill_records: 1,
            extra_statistics: 0,
            event_gap: 0,
            compact_results: 0,
            invisible_back: 0,
            auto_hill_record_replay: 1,
            sound_effects: 0,
            unique_computer_names: 0,
            goals_enabled: 1,
            wc_gap: 0,
            ko_system: 1,
            language: 255,
            training_rounds: 0,
            name_set_index: 0,
            last_custom_cup_file: String::from("TEMP"),
            graphics_detail: 0,
            visible_computers: 240,
            koth_wind: 0,
            koth_rounds: 2,
            koth_pack: 1,
            koth_hill: -1,
            koth_opponent_count: 1,
            koth_opponent_ids: vec![0],
            key_up: 72,
            key_right: 77,
            key_left: 75,
            key_telemark: 21524,
            key_replay: 21011,
            wind_position: 0,
        }
    }
}

impl Config {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, String> {
        let file: ConfigFile = parse_toml(data)?;
        if file.format_version != FORMAT_VERSION {
            return Err(format!("unsupported config format {}", file.format_version));
        }
        let config = file.config;
        config.validate()?;
        Ok(config)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, String> {
        let file = ConfigFile {
            format_version: FORMAT_VERSION,
            config: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|error| error.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        let toggles = [
            ("computer_hill_records", self.computer_hill_records),
            ("extra_statistics", self.extra_statistics),
            ("event_gap", self.event_gap),
            ("compact_results", self.compact_results),
            ("invisible_back", self.invisible_back),
            ("auto_hill_record_replay", self.auto_hill_record_replay),
            ("sound_effects", self.sound_effects),
            ("unique_computer_names", self.unique_computer_names),
            ("goals_enabled", self.goals_enabled),
            ("wc_gap", self.wc_gap),
            ("ko_system", self.ko_system),
            ("graphics_detail", self.graphics_detail),
        ];
        if let Some((name, value)) = toggles
            .into_iter()
            .find(|(_, value)| !(0..=1).contains(value))
        {
            return Err(format!("{name} must be 0 or 1, got {value}"));
        }
        let ranges = [
            ("language", self.language, 0, 255),
            ("training_rounds", self.training_rounds, 0, 3),
            ("name_set_index", self.name_set_index, 0, 255),
            ("visible_computers", self.visible_computers, 0, 240),
            ("koth_wind", self.koth_wind, 0, 2),
            ("koth_rounds", self.koth_rounds, 1, 2),
            ("koth_pack", self.koth_pack, 0, 255),
            ("koth_hill", self.koth_hill, -1, 10_000),
            ("koth_opponent_count", self.koth_opponent_count, 1, 20),
            (
                "wind_position",
                self.wind_position,
                0,
                i32::from(WIND_POSITION_COUNT) - 1,
            ),
        ];
        if let Some((name, value, min, max)) = ranges
            .into_iter()
            .find(|(_, value, min, max)| value < min || value > max)
        {
            return Err(format!("{name} must be in {min}..={max}, got {value}"));
        }
        if self.koth_opponent_ids.len() != self.koth_opponent_count as usize
            || self
                .koth_opponent_ids
                .iter()
                .any(|&id| !(0..=10_000).contains(&id))
        {
            return Err("koth opponent vector does not match its count".into());
        }
        if [
            self.key_up,
            self.key_right,
            self.key_left,
            self.key_telemark,
            self.key_replay,
        ]
        .into_iter()
        .any(|key| key < 0)
        {
            return Err("key bindings must be non-negative".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_wind_position_outside_zero_based_range_is_rejected() {
        let mut config = Config {
            wind_position: -1,
            ..Config::default()
        };
        let Ok(bytes) = config.to_toml_bytes() else {
            return;
        };
        assert!(Config::from_toml_bytes(&bytes).is_err());

        config.wind_position = i32::from(WIND_POSITION_COUNT);
        let Ok(bytes) = config.to_toml_bytes() else {
            return;
        };
        assert!(Config::from_toml_bytes(&bytes).is_err());
    }

    #[test]
    fn koth_ranges_match_game_limits() {
        let mut config = Config::default();

        for rounds in [1, 2] {
            config.koth_rounds = rounds;
            assert!(config.validate().is_ok());
        }
        for rounds in [0, 3] {
            config.koth_rounds = rounds;
            assert!(config.validate().is_err());
        }

        config.koth_rounds = 2;
        for count in [1, 20] {
            config.koth_opponent_count = count;
            config.koth_opponent_ids = vec![0; count as usize];
            assert!(config.validate().is_ok());
        }
        for count in [0, 21] {
            config.koth_opponent_count = count;
            config.koth_opponent_ids = vec![0; count as usize];
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn obsolete_profile_fields_are_not_persisted() {
        let Ok(bytes) = Config::default().to_toml_bytes() else {
            return;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return;
        };
        assert!(!text.contains("jumper_count"));
        assert!(!text.contains("jumper_names"));
        assert!(!text.contains("player_count"));
        assert!(!text.contains("profile_order"));
    }
}
