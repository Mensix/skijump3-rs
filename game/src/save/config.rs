use serde::{Deserialize, Serialize};

/// TOML wrapper to version the file.
#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    format_version: u32,
    #[serde(flatten)]
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub reg: i32,
    pub comphrs: i32,
    pub lct: i32,
    pub diff: i32,
    pub compactlist: i32,
    pub invback: i32,
    pub automatichrr: i32,
    pub beeppi: i32,
    pub nosamename: i32,
    pub goals: i32,
    pub diffwc: i32,
    pub kosystem: i32,
    pub languagenumber: i32,
    pub trainrounds: i32,
    pub namenumber: i32,
    pub setfile: String,
    pub gdetail: i32,
    pub seecomps: i32,
    pub jumper_count: i32,
    pub jnimet: Vec<String>,
    pub player_count: i32,
    pub profileorder: Vec<i32>,
    pub kothwind: i32,
    pub kothrounds: i32,
    pub kothpack: i32,
    pub kothmaki: i32,
    pub koth_count: i32,
    pub kothpel: Vec<i32>,
    pub key_up: i32,
    pub key_right: i32,
    pub key_left: i32,
    pub key_telemark: i32,
    pub key_replay: i32,
    pub windplace: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reg: 1,
            comphrs: 1,
            lct: 0,
            diff: 0,
            compactlist: 0,
            invback: 0,
            automatichrr: 0,
            beeppi: 0,
            nosamename: 0,
            goals: 0,
            diffwc: 0,
            kosystem: 1,
            languagenumber: 255,
            trainrounds: 0,
            namenumber: 0,
            setfile: String::from("TEMP"),
            gdetail: 0,
            seecomps: 240,
            jumper_count: 1,
            jnimet: vec![String::from("A TEAM")],
            player_count: 1,
            profileorder: vec![1],
            kothwind: 0,
            kothrounds: 2,
            kothpack: 1,
            kothmaki: 0,
            koth_count: 1,
            kothpel: vec![1],
            key_up: 72,
            key_right: 77,
            key_left: 75,
            key_telemark: 21524,
            key_replay: 21011,
            windplace: 1,
        }
    }
}

impl Config {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, String> {
        let text =
            std::str::from_utf8(data).map_err(|e| format!("Invalid UTF-8 in config: {e}"))?;
        let file: ConfigFile =
            toml::from_str(text).map_err(|e| format!("Failed to parse config: {e}"))?;
        if file.format_version != 1 {
            return Err(format!(
                "Unsupported config format_version: {}",
                file.format_version
            ));
        }
        Ok(file.config)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, String> {
        let file = ConfigFile {
            format_version: 1,
            config: self.clone(),
        };
        toml::to_string(&file)
            .map(|s| s.into_bytes())
            .map_err(|e| format!("Failed to serialize config: {e}"))
    }
}
