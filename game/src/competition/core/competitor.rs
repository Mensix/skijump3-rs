use crate::data::profile::{Profile, ProfileStore};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competitor {
    pub id: usize,
    pub ai_id: usize,
    pub name: String,
    pub real_name: String,
    pub suit_color: [u8; 3],
    pub ski_color: [u8; 3],
    pub team: Option<usize>,
    pub is_computer: bool,
    pub profile_idx: Option<usize>,
}

impl Competitor {
    pub fn computer(id: usize, ai_id: usize, name: String, team: Option<usize>) -> Self {
        Self {
            id,
            ai_id,
            name,
            real_name: String::new(),
            suit_color: [0, 0, 0],
            ski_color: [0, 0, 0],
            team,
            is_computer: true,
            profile_idx: None,
        }
    }

    pub fn from_profile(
        id: usize,
        profile_idx: usize,
        profile: &Profile,
        team: Option<usize>,
    ) -> Self {
        Self {
            id,
            ai_id: 0,
            name: profile.name.clone(),
            real_name: profile.real_name.clone(),
            suit_color: profile.suit_color,
            ski_color: profile.ski_color,
            team,
            is_computer: false,
            profile_idx: Some(profile_idx),
        }
    }

    pub fn to_jump_participant(&self) -> JumpParticipant {
        JumpParticipant {
            id: self.id,
            ai_id: self.ai_id,
            name: self.name.clone(),
            real_name: self.real_name.clone(),
            suit_color: self.suit_color,
            ski_color: self.ski_color,
            team: self.team,
            control: if self.is_computer {
                JumperControl::Computer
            } else {
                JumperControl::Human
            },
        }
    }
}

pub fn active_profiles(profiles: &ProfileStore) -> Vec<(usize, &Profile)> {
    profiles
        .active_order
        .iter()
        .copied()
        .filter_map(|idx| Some((idx, profiles.profiles.get(idx)?)))
        .collect()
}

pub fn computer_names_without_replacements(
    computer_names: &[String],
    active_profiles: &[(usize, &Profile)],
) -> Vec<String> {
    computer_names
        .iter()
        .enumerate()
        .filter(|(idx, _)| {
            let replace = *idx;
            !active_profiles
                .iter()
                .any(|(_, profile)| profile.replace == Some(replace))
        })
        .map(|(_, name)| name.clone())
        .collect()
}
