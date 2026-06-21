use crate::competition::active::{ActiveCompetition, ActiveCompetitionKind};
use crate::save::parse_toml;
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct CupSaveEntry {
    pub filename: String,
    pub title: String,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CupSaveData {
    pub saved_at: String,
    pub active: ActiveCompetition,
}

#[derive(Debug, Serialize, Deserialize)]
struct CupSaveFile {
    format_version: u32,
    #[serde(flatten)]
    data: CupSaveData,
}

impl CupSaveData {
    pub fn new(active: ActiveCompetition, saved_at: String) -> Self {
        Self { saved_at, active }
    }

    pub fn from_toml_bytes(data: &[u8]) -> Self {
        let file: CupSaveFile = parse_toml(data);
        assert_eq!(
            file.format_version, FORMAT_VERSION,
            "unsupported cup save format {}",
            file.format_version
        );
        file.data
    }

    pub fn to_toml_bytes(&self) -> Vec<u8> {
        let file = CupSaveFile {
            format_version: FORMAT_VERSION,
            data: self.clone(),
        };
        toml::to_string_pretty(&file).unwrap().into_bytes()
    }

    pub fn title(&self) -> String {
        match self.active.kind() {
            ActiveCompetitionKind::Training => "Training".to_string(),
            ActiveCompetitionKind::Individual => self
                .active
                .individual()
                .map(|comp| format!("{:?} event {}", comp.style(), comp.current_event + 1))
                .unwrap_or_else(|| "Individual Cup".to_string()),
            ActiveCompetitionKind::TeamCup => "Team Cup".to_string(),
            ActiveCompetitionKind::Koth => "King of the Hill".to_string(),
        }
    }
}
