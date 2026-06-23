use crate::data::profile::Profile;

#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub name: String,
    pub ready: bool,
    pub is_host: bool,
    pub suit_color: [u8; 3],
    pub ski_color: [u8; 3],
}

impl LobbyPlayer {
    pub fn from_profile(_profile_idx: usize, p: &Profile, is_host: bool) -> Self {
        Self {
            name: p.name.clone(),
            ready: false,
            is_host,
            suit_color: p.suit_color,
            ski_color: p.ski_color,
        }
    }

    pub fn empty_slot() -> Self {
        Self {
            name: String::new(),
            ready: false,
            is_host: false,
            suit_color: [0; 3],
            ski_color: [0; 3],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}

pub const MAX_PLAYERS: usize = 10;

#[derive(Debug, Clone)]
pub struct LobbyState {
    pub players: Vec<LobbyPlayer>,
    pub is_host: bool,
    pub local_idx: usize,
    pub hill_idx: usize,
}

impl LobbyState {
    pub fn new(is_host: bool, local_player: LobbyPlayer) -> Self {
        let mut players = vec![local_player];
        for _ in 1..MAX_PLAYERS {
            players.push(LobbyPlayer::empty_slot());
        }
        Self {
            players,
            is_host,
            local_idx: 0,
            hill_idx: 0,
        }
    }

    pub fn connected_count(&self) -> usize {
        self.players.iter().filter(|p| !p.is_empty()).count()
    }
}
