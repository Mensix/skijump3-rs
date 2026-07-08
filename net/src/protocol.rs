use serde::{Deserialize, Serialize};

pub const PORT: u16 = 34503;
pub const DISCOVERY_PORT: u16 = 34504;
pub const DISCOVERY_MAGIC: &str = "SJ3_DISCOVER";
pub const DISCOVERY_RESPONSE: &str = "SJ3_ROOM";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub name: String,
    pub suit: [u8; 3],
    pub ski: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello(Hello),
    Ready(bool),
    Chat(String),
    Leave,
    JumpComplete {
        distance: f64,
        score: f64,
        style_points: [f64; 5],
        landing_style: u8,
        fall_type: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: usize,
    pub name: String,
    pub ready: bool,
    pub is_host: bool,
    pub suit: [u8; 3],
    pub ski: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbySnapshot {
    pub players: Vec<PlayerInfo>,
    pub hill_idx: usize,
    pub total_legs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub from: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpRound {
    pub hill_idx: usize,
    pub round: usize,
    pub wind_seed: u32,
    pub wind_position: u8,
    pub start_gate: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MPStandingEntry {
    pub player_id: usize,
    pub name: String,
    pub round1_len: f64,
    pub round1_score: f64,
    pub round2_len: f64,
    pub round2_score: f64,
    pub total_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome {
        id: usize,
    },
    Lobby(LobbySnapshot),
    Chat(ChatMsg),
    JumpRound(JumpRound),
    StandingsUpdate {
        round: usize,
        entries: Vec<MPStandingEntry>,
    },
    CompetitionDone {
        entries: Vec<MPStandingEntry>,
    },
    Error(String),
}

#[derive(Debug)]
pub enum NetEvent {
    ClientHello(usize, Hello),
    ClientReady(usize, bool),
    ClientChat(usize, String),
    ClientLeft(usize),
    ClientJumpComplete {
        id: usize,
        distance: f64,
        score: f64,
        style_points: [f64; 5],
        landing_style: u8,
        fall_type: u8,
    },
    Connected,
    ServerMsg(ServerMsg),
    Disconnected,
    Error(String),
}
