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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub from: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Start {
    pub hill_idx: usize,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { id: usize },
    Lobby(LobbySnapshot),
    Chat(ChatMsg),
    Start(Start),
    Error(String),
}

#[derive(Debug)]
pub enum NetEvent {
    ClientHello(usize, Hello),
    ClientReady(usize, bool),
    ClientChat(usize, String),
    ClientLeft(usize),
    Connected,
    ServerMsg(ServerMsg),
    Disconnected,
    Error(String),
}
