use crate::data::profile::Profile;

#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    #[allow(dead_code)]
    pub profile_idx: usize,
    pub name: String,
    pub ready: bool,
    pub is_host: bool,
    #[allow(dead_code)]
    pub suit_color: [u8; 3],
    #[allow(dead_code)]
    pub ski_color: [u8; 3],
}

impl LobbyPlayer {
    pub fn from_profile(profile_idx: usize, p: &Profile, is_host: bool) -> Self {
        Self {
            profile_idx,
            name: p.name.clone(),
            ready: false,
            is_host,
            suit_color: p.suit_color,
            ski_color: p.ski_color,
        }
    }

    pub fn empty_slot() -> Self {
        Self {
            profile_idx: usize::MAX,
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
const CHAT_HISTORY: usize = 50;

#[derive(Debug, Clone)]
pub struct ChatBuffer {
    pub messages: Vec<(String, String)>,
}

impl ChatBuffer {
    pub fn new() -> Self {
        Self {
            messages: Vec::with_capacity(CHAT_HISTORY),
        }
    }

    pub fn push(&mut self, sender: String, text: String) {
        if self.messages.len() >= CHAT_HISTORY {
            self.messages.remove(0);
        }
        self.messages.push((sender, text));
    }

    pub fn send(&mut self, text: String) {
        self.push("You".to_string(), text);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyPhase {
    Setup,
    #[allow(dead_code)]
    Starting,
}

#[derive(Debug, Clone)]
pub struct LobbyState {
    pub players: Vec<LobbyPlayer>,
    pub is_host: bool,
    pub local_idx: usize,
    #[allow(dead_code)]
    pub hill_idx: usize,
    #[allow(dead_code)]
    pub phase: LobbyPhase,
    pub chat: ChatBuffer,
}

impl LobbyState {
    pub fn new(is_host: bool, local_player: LobbyPlayer) -> Self {
        let local_idx = 0;
        let mut players = vec![local_player];
        for _ in 1..MAX_PLAYERS {
            players.push(LobbyPlayer::empty_slot());
        }
        Self {
            players,
            is_host,
            local_idx,
            hill_idx: 0,
            phase: LobbyPhase::Setup,
            chat: ChatBuffer::new(),
        }
    }

    #[allow(dead_code)]
    pub fn local(&self) -> &LobbyPlayer {
        &self.players[self.local_idx]
    }

    #[allow(dead_code)]
    pub fn local_mut(&mut self) -> &mut LobbyPlayer {
        &mut self.players[self.local_idx]
    }

    pub fn toggle_ready(&mut self) {
        self.players[self.local_idx].ready = !self.players[self.local_idx].ready;
    }

    #[allow(dead_code)]
    pub fn connected_count(&self) -> usize {
        self.players.iter().filter(|p| !p.is_empty()).count()
    }

    pub fn all_ready(&self) -> bool {
        let connected: Vec<_> = self.players.iter().filter(|p| !p.is_empty()).collect();
        !connected.is_empty() && connected.iter().all(|p| p.ready)
    }
}
