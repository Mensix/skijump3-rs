use serde::{Deserialize, Serialize};

use crate::save::parse_toml;
use crate::save::SaveError;

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
            reg: 0,
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
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, SaveError> {
        let file: ConfigFile = parse_toml(data)?;
        if file.format_version != 1 {
            return Err(SaveError::Serialization(format!(
                "Unsupported config format_version: {}",
                file.format_version
            )));
        }
        Ok(file.config)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, SaveError> {
        let file = ConfigFile {
            format_version: 1,
            config: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|e| SaveError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_all_fields() {
        let cfg = Config::default();
        let bytes = cfg.to_toml_bytes().unwrap();
        let parsed = Config::from_toml_bytes(&bytes).unwrap();
        assert_eq!(parsed.reg, cfg.reg);
        assert_eq!(parsed.comphrs, cfg.comphrs);
        assert_eq!(parsed.lct, cfg.lct);
        assert_eq!(parsed.diff, cfg.diff);
        assert_eq!(parsed.compactlist, cfg.compactlist);
        assert_eq!(parsed.invback, cfg.invback);
        assert_eq!(parsed.automatichrr, cfg.automatichrr);
        assert_eq!(parsed.beeppi, cfg.beeppi);
        assert_eq!(parsed.nosamename, cfg.nosamename);
        assert_eq!(parsed.goals, cfg.goals);
        assert_eq!(parsed.diffwc, cfg.diffwc);
        assert_eq!(parsed.kosystem, cfg.kosystem);
        assert_eq!(parsed.languagenumber, cfg.languagenumber);
        assert_eq!(parsed.trainrounds, cfg.trainrounds);
        assert_eq!(parsed.namenumber, cfg.namenumber);
        assert_eq!(parsed.setfile, cfg.setfile);
        assert_eq!(parsed.gdetail, cfg.gdetail);
        assert_eq!(parsed.seecomps, cfg.seecomps);
        assert_eq!(parsed.jumper_count, cfg.jumper_count);
        assert_eq!(parsed.jnimet, cfg.jnimet);
        assert_eq!(parsed.player_count, cfg.player_count);
        assert_eq!(parsed.profileorder, cfg.profileorder);
        assert_eq!(parsed.kothwind, cfg.kothwind);
        assert_eq!(parsed.kothrounds, cfg.kothrounds);
        assert_eq!(parsed.kothpack, cfg.kothpack);
        assert_eq!(parsed.kothmaki, cfg.kothmaki);
        assert_eq!(parsed.koth_count, cfg.koth_count);
        assert_eq!(parsed.kothpel, cfg.kothpel);
        assert_eq!(parsed.key_up, cfg.key_up);
        assert_eq!(parsed.key_right, cfg.key_right);
        assert_eq!(parsed.key_left, cfg.key_left);
        assert_eq!(parsed.key_telemark, cfg.key_telemark);
        assert_eq!(parsed.key_replay, cfg.key_replay);
        assert_eq!(parsed.windplace, cfg.windplace);
    }

    #[test]
    fn rejects_bad_format_version() {
        let bytes = b"format_version = 99\nreg = 0\n";
        let result = Config::from_toml_bytes(bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("format_version"));
    }

    #[test]
    fn rejects_invalid_toml() {
        let bytes = b"garbage [[[toml]]]\n";
        let result = Config::from_toml_bytes(bytes);
        assert!(result.is_err());
    }
}
