use crate::competition::ActiveCompetition;
use crate::content::hills::load_hills;
use crate::content::names::NameCatalog;
use crate::data::hill::HillCatalog;
use crate::data::hill_profile::{stored_profile_matches, HillProfileMismatch, HillTerrain};
use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::jump::types::DEFAULT_START_GATE;
use crate::jump::wind::Wind;
use crate::rng::Random;
use crate::save::config::Config;
use crate::text::lang::LangBase;
use crate::ui::Font;
use engine::audio::Beep;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Font,
    pub langbase: Rc<LangBase>,
    pub namesets: NameCatalog,
    pub hills: HillCatalog,
    pub(crate) terrain_cache: RefCell<TerrainCache>,
    pub files: Rc<FileStore>,
}

impl Resources {
    pub fn new(
        font: Font,
        langbase: Rc<LangBase>,
        namesets: NameCatalog,
        hills: HillCatalog,
        files: Rc<FileStore>,
    ) -> Self {
        Self {
            font,
            langbase,
            namesets,
            hills,
            terrain_cache: RefCell::new(TerrainCache::new(4)),
            files,
        }
    }

    pub fn player_names(&self, name_set_index: usize) -> &[String] {
        self.namesets.names_for_config(name_set_index as i32)
    }

    pub(crate) fn terrain(&self, hill_idx: usize) -> Rc<HillTerrain> {
        let Some(hill) = self.hills.hill(hill_idx).map(|hill| hill.clone()) else {
            return Rc::new(HillTerrain::default());
        };
        let cache_key = format!(
            "{}:{}:{}:{}:{}:{}",
            hill.record_key,
            hill.front_index,
            hill.back_index,
            hill.back_brightness,
            hill.back_mirror,
            hill.profile_checksum
        );
        let mut cache = self.terrain_cache.borrow_mut();
        if let Some(terrain) = cache.get(&cache_key) {
            return terrain;
        }
        let terrain = HillTerrain::load_with_markers(
            &self.files,
            &hill.front_index,
            &hill.back_index,
            hill.back_brightness,
            hill.back_mirror != 0,
            hill.kr,
            hill.pk(),
        );
        let terrain = Rc::new(terrain);
        cache.insert(cache_key, terrain.clone());
        terrain
    }

    pub(crate) fn refresh_hills(&self) {
        let Ok(hills) = load_hills(&self.files, "hills/manifest.toml") else {
            return;
        };
        self.hills.replace(hills);
        self.terrain_cache.borrow_mut().clear();
    }

    pub(crate) fn verify_hill_profile(&self, hill_idx: usize) -> Option<HillProfileMismatch> {
        let hill = self.hills.hill(hill_idx)?;
        if stored_profile_matches(&self.files, &hill.front_index, hill.profile_checksum) {
            return None;
        }
        Some(HillProfileMismatch {
            front_index: hill.front_index.clone(),
            exiting_cup: hill_idx < self.hills.original_count(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TerrainCache {
    entries: HashMap<String, Rc<HillTerrain>>,
    lru: VecDeque<String>,
    capacity: usize,
}

impl TerrainCache {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru: VecDeque::new(),
            capacity,
        }
    }

    fn get(&mut self, key: &str) -> Option<Rc<HillTerrain>> {
        let value = self.entries.get(key)?.clone();
        self.lru.retain(|cached| cached != key);
        self.lru.push_back(key.to_string());
        Some(value)
    }

    fn insert(&mut self, key: String, terrain: Rc<HillTerrain>) {
        self.lru.retain(|cached| cached != &key);
        self.entries.insert(key.clone(), terrain);
        self.lru.push_back(key);
        while self.entries.len() > self.capacity {
            if let Some(oldest) = self.lru.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.lru.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub type ResourcesRef = Rc<Resources>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindPolicy {
    Enabled,
    Disabled,
}

impl WindPolicy {
    const fn for_new(comp: &ActiveCompetition, config: &Config) -> Self {
        if matches!(comp, ActiveCompetition::Koth(_)) && config.koth_wind == 0 {
            Self::Disabled
        } else {
            Self::Enabled
        }
    }

    const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub config: Config,
    pub rng: Random,
    pub wind: Wind,
    pub first_event: bool,
    pub practice_hill: usize,
    pub practice_start_gate: i32,
    pub active_competition: Option<ActiveCompetition>,
    pub profiles: ProfileStore,
    pub records: RecordStore,
    audio_cues: VecDeque<Beep>,
}

impl GameState {
    const MAX_AUDIO_CUES: usize = 8;
    pub fn new(records: RecordStore, profiles: ProfileStore, config: Config) -> Self {
        Self {
            config,
            rng: Random::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as u32,
            ),
            wind: Wind::default(),
            first_event: true,
            practice_hill: 0,
            practice_start_gate: DEFAULT_START_GATE,
            active_competition: None,
            profiles,
            records,
            audio_cues: VecDeque::new(),
        }
    }

    pub fn start_active(&mut self, comp: ActiveCompetition) {
        let wind_policy = WindPolicy::for_new(&comp, &self.config);
        self.activate_competition(comp, wind_policy);
    }

    pub fn abort_active_competition(&mut self) {
        self.active_competition = None;
    }

    fn activate_competition(&mut self, comp: ActiveCompetition, wind_policy: WindPolicy) {
        self.wind.set_enabled(wind_policy.is_enabled());
        self.active_competition = Some(comp);
    }

    pub fn consume_first_jump_event(&mut self) -> bool {
        let prev = self.first_event;
        self.first_event = false;
        prev
    }

    pub fn reset_practice_wind(&mut self) {
        self.wind
            .initialize(&mut self.rng, self.config.wind_position as u8);
    }

    pub(crate) fn request_beep(&mut self, beep: Beep) {
        if self.config.sound_effects != 0 && self.audio_cues.len() < Self::MAX_AUDIO_CUES {
            self.audio_cues.push_back(beep);
        }
    }

    pub(crate) fn drain_audio_cues(&mut self) -> impl Iterator<Item = Beep> + '_ {
        self.audio_cues.drain(..)
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(
            RecordStore::default(),
            ProfileStore::default(),
            Config::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::koth::types::{KothPhase, KothRuntime};
    use crate::competition::machine::Competition;
    use crate::competition::team_cup::types::TeamCupRuntime;
    use crate::competition::types::CupStyle;

    fn koth() -> ActiveCompetition {
        ActiveCompetition::Koth(KothRuntime {
            participants: vec![],
            human_indices: vec![],
            hill_idx: 0,
            jump_rounds_per_elimination: 1,
            phase: KothPhase::Setup,
            current_elimination_round: 0,
            current_jump_round: 0,
            current_participant_pos: 0,
            rng: Random::new(0),
        })
    }

    fn team_cup() -> ActiveCompetition {
        ActiveCompetition::TeamCup(TeamCupRuntime::new(vec![], vec![], vec![]))
    }

    #[test]
    fn sound_config_gates_cues() {
        let mut state = GameState::default();
        state.config.sound_effects = 0;
        state.request_beep(Beep::Type1);
        assert_eq!(state.drain_audio_cues().count(), 0);

        state.config.sound_effects = 1;
        state.request_beep(Beep::Type2);
        assert_eq!(state.drain_audio_cues().collect::<Vec<_>>(), [Beep::Type2]);
    }

    #[test]
    fn new_competitions_apply_mode_wind_policy() {
        let mut state = GameState::default();
        state.config.koth_wind = 0;
        state.start_active(ActiveCompetition::Training);
        assert!(state.wind.is_enabled());

        state.start_active(ActiveCompetition::Individual(Competition::new(
            CupStyle::WorldCup,
            vec![],
            vec![],
        )));
        assert!(state.wind.is_enabled());

        state.start_active(team_cup());
        assert!(state.wind.is_enabled());

        state.start_active(koth());
        assert!(!state.wind.is_enabled());
    }

    #[test]
    fn audio_cue_backlog_is_bounded() {
        let mut state = GameState::default();
        state.config.sound_effects = 1;
        for _ in 0..100 {
            state.request_beep(Beep::Type1);
        }
        assert_eq!(state.drain_audio_cues().count(), GameState::MAX_AUDIO_CUES);
    }

    #[test]
    fn terrain_cache_is_bounded_and_promotes_hits() {
        let files = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from("."),
        );
        let terrain = Rc::new(HillTerrain::load_with_markers(
            &files, "1", "1", 100, false, 0, 0.0,
        ));
        let mut cache = TerrainCache::new(2);
        cache.insert("a".to_string(), terrain.clone());
        cache.insert("b".to_string(), terrain.clone());
        assert!(cache.get("a").is_some());

        cache.insert("c".to_string(), terrain);

        assert_eq!(cache.len(), 2);
        assert!(cache.entries.contains_key("a"));
        assert!(!cache.entries.contains_key("b"));
        assert!(cache.entries.contains_key("c"));
    }
}
