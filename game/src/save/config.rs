use serde::{Deserialize, Serialize};

use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    format_version: u32,
    #[serde(flatten)]
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub jumper_count: i32,
    pub jumper_names: Vec<String>,
    pub player_count: i32,
    pub profile_order: Vec<i32>,
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
            visible_computers: 0,
            jumper_count: 1,
            jumper_names: vec![String::from("A TEAM")],
            player_count: 1,
            profile_order: vec![1],
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
    pub fn from_toml_bytes(data: &[u8]) -> Self {
        let file: ConfigFile = parse_toml(data);
        file.config
    }

    pub fn to_toml_bytes(&self) -> Vec<u8> {
        let file = ConfigFile {
            format_version: 1,
            config: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .unwrap()
    }
}
