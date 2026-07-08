use std::net::SocketAddr;
use std::time::Duration;

use engine::oxide::input::{Key, UiEvent};
use engine::oxide::paint::PaintCx;
use engine::oxide::{ScreenBackground, ScreenEventCx};
use net::client::ClientHandle;
use net::discovery;
use net::host::HostHandle;
use net::protocol::{
    self, ChatMsg, ClientMsg, Hello, JumpRound, LobbySnapshot, MPStandingEntry, NetEvent,
    PlayerInfo, ServerMsg,
};

use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::multiplayer::runtime::MultiplayerRuntime;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::views::multiplayer::chat::{ChatAction, ChatPanel};
use crate::views::multiplayer::state::{LobbyPlayer, LobbyState};

pub struct MultiplayerLobbyView {
    phase: LobbyPhase,
    status: String,
    chat: ChatPanel,
    pending_start: bool,
}

enum LobbyPhase {
    Connected,
    Failed,
}

impl MultiplayerLobbyView {
    pub fn new() -> Self {
        Self {
            phase: LobbyPhase::Connected,
            status: String::new(),
            chat: ChatPanel::default(),
            pending_start: false,
        }
    }
}

impl GameScreen for MultiplayerLobbyView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        if self.status.is_empty() {
            let discovered = discovery::discover(Duration::ZERO).ok().flatten();
            let joined = if let Some(addr) = discovered {
                self.join_host(cx, addr)
            } else {
                false
            };
            if joined || self.join_localhost(cx) || self.become_host(cx) {
                self.phase = LobbyPhase::Connected;
            } else {
                self.phase = LobbyPhase::Failed;
            }
        }

        if let LobbyPhase::Connected = self.phase {
            self.chat.tick();
            self.poll_events(cx);
            if self.pending_start {
                self.pending_start = false;
                self.start_mp(cx);
            }
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let LobbyPhase::Failed = self.phase {
            if let UiEvent::KeyDown(Key::Escape) = event {
                if let Some(ref host) = cx.state.net_host {
                    host.stop();
                }
                cx.state.net_host.take();
                cx.state.net_client.take();
                nav.back();
                nav.consume();
            }
            return;
        }

        if self.pending_start {
            self.pending_start = false;
            self.do_start_mp(cx, nav);
        }

        match event {
            UiEvent::KeyDown(Key::Escape) => {
                if !self.chat.input.is_empty() {
                    self.chat.input.clear();
                } else {
                    if let Some(ref host) = cx.state.net_host {
                        host.stop();
                    }
                    cx.state.net_host.take();
                    cx.state.net_client.take();
                    nav.back();
                }
                nav.consume();
                return;
            }
            UiEvent::KeyDown(Key::F1) => {
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    lobby.players[lobby.local_idx].ready = !lobby.players[lobby.local_idx].ready;
                    let should_start = cx.state.net_host.is_some() && all_ready(lobby);
                    if let Some(ref host) = cx.state.net_host {
                        let snapshot = snapshot_from_lobby(lobby);
                        host.broadcast(ServerMsg::Lobby(snapshot));
                    }
                    if let Some(ref client) = cx.state.net_client {
                        let ready = lobby.players[lobby.local_idx].ready;
                        client.send(ClientMsg::Ready(ready));
                    }
                    if should_start {
                        self.pending_start = true;
                    }
                }
                nav.consume();
                return;
            }
            UiEvent::KeyDown(Key::F2) => {
                if cx.state.net_host.is_some() {
                    if let Some(ref lobby) = cx.state.pending_lobby {
                        let all_ready = all_ready(lobby);
                        if all_ready {
                            self.do_start_mp(cx, nav);
                            return;
                        }
                    }
                }
                nav.consume();
            }
            _ => {}
        }

        match self.chat.event(event) {
            ChatAction::Send(text) => {
                self.send_chat(cx, &text);
                nav.consume();
            }
            ChatAction::Consumed => {
                nav.consume();
            }
            ChatAction::None => {}
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        paint.pattern_fill((0, 20, 320, 180), BG_PURPLE);
        paint.sprite(sprites::Sprite::Logo as u16, (5, 2));

        paint.text((30, 6), FONT_BODY, "MULTIPLAYER ROOM");

        match self.phase {
            LobbyPhase::Failed => {
                paint.center_text((160, 90), FONT_GOLD, &self.status);
                paint.center_text((160, 102), FONT_GRAY, "Esc leave");
            }
            LobbyPhase::Connected => {
                if self.status.is_empty() {
                    paint.center_text((160, 90), FONT_GRAY, "Connecting...");
                    return;
                }

                paint.right_text((316, 5), FONT_GRAY, "F1 rd  F2 go)");
                paint.right_text((316, 13), FONT_GRAY, "Esc leave)");

                if let Some(ref lobby) = cx.state.pending_lobby {
                    for (i, p) in lobby.players.iter().enumerate() {
                        if p.is_empty() {
                            continue;
                        }
                        let y = 23 + (i as i32) * 8;
                        let is_local = i == lobby.local_idx;
                        paint.right_text((24, y), FONT_GOLD, format!("{}.", i + 1));
                        paint.text(
                            (32, y),
                            if is_local { FONT_GOLD } else { FONT_BODY },
                            &p.name,
                        );
                        paint.right_text(
                            (184, y),
                            if p.ready {
                                if is_local {
                                    FONT_GOLD
                                } else {
                                    FONT_TEAL
                                }
                            } else {
                                FONT_GRAY
                            },
                            if p.ready { "RDY" } else { "AWAITING" },
                        );
                    }
                }

                let p = &cx.state.profiles.profiles[0];
                let display = if p.real_name.is_empty() {
                    &p.name
                } else {
                    &p.real_name
                };
                self.chat.paint(paint, 143, display);
            }
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

impl MultiplayerLobbyView {
    fn become_host(&mut self, cx: &mut GameCx<'_>) -> bool {
        match HostHandle::start() {
            Ok(host) => {
                cx.state.net_host = Some(host);
                let p = &cx.state.profiles.profiles[0];
                let local = LobbyPlayer::from_profile(0, p, true);
                cx.state.pending_lobby = Some(LobbyState::new(true, local));
                self.status = "hosting room".to_string();
                true
            }
            Err(e) => {
                self.status = format!("host failed: {e}");
                false
            }
        }
    }

    fn join_localhost(&mut self, cx: &mut GameCx<'_>) -> bool {
        self.join_host(cx, SocketAddr::from(([127, 0, 0, 1], protocol::PORT)))
    }

    fn join_host(&mut self, cx: &mut GameCx<'_>, addr: SocketAddr) -> bool {
        let p = &cx.state.profiles.profiles[0];
        let hello = Hello {
            name: p.name.clone(),
            suit: p.suit_color,
            ski: p.ski_color,
        };
        match ClientHandle::connect(addr, hello) {
            Ok(client) => {
                cx.state.net_client = Some(client);
                let local = LobbyPlayer::from_profile(0, p, false);
                cx.state.pending_lobby = Some(LobbyState::new(false, local));
                self.status = format!("joined {addr}");
                true
            }
            Err(e) => {
                self.status = format!("join {addr} failed: {e}");
                false
            }
        }
    }

    fn send_chat(&mut self, cx: &mut GameCx<'_>, text: &str) {
        let my_name = cx.state.profiles.profiles[0].name.clone();
        if let Some(ref client) = cx.state.net_client {
            client.send(ClientMsg::Chat(text.to_string()));
        }
        if cx.state.net_host.is_some() {
            self.chat.push_message(my_name.clone(), text.to_string());
            let msg = ServerMsg::Chat(ChatMsg {
                from: my_name,
                text: text.to_string(),
            });
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(msg);
            }
        }
    }

    fn do_start_mp(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>) {
        if self.start_mp(cx) {
            nav.navigate(RouteTarget::MultiplayerJump);
        }
    }

    fn start_mp(&mut self, cx: &mut GameCx<'_>) -> bool {
        let Some(ref lobby) = cx.state.pending_lobby else {
            return false;
        };
        let seed = cx.state.rng.random_i32(i32::MAX) as u32;
        let pos = cx.state.config.wind_position as u8;
        cx.state.wind.initialize(&mut cx.state.rng, pos);
        let mut entries = Vec::new();
        for (i, p) in lobby.players.iter().enumerate() {
            if p.is_empty() {
                continue;
            }
            entries.push(MPStandingEntry {
                player_id: i,
                name: p.name.clone(),
                round1_len: 0.0,
                round1_score: 0.0,
                round2_len: 0.0,
                round2_score: 0.0,
                total_points: 0.0,
            });
        }
        let hill_idx = lobby.hill_idx;
        let round = JumpRound {
            hill_idx,
            round: 0,
            wind_seed: seed,
            wind_position: pos,
            start_gate: 15,
        };
        if let Some(ref host) = cx.state.net_host {
            host.broadcast(ServerMsg::JumpRound(round.clone()));
            host.broadcast(ServerMsg::StandingsUpdate {
                round: 0,
                entries: entries.clone(),
            });
        }
        cx.state.mp_jump = Some(MultiplayerRuntime::new(
            hill_idx,
            lobby.total_legs,
            15,
            entries,
            0,
        ));
        cx.state.pending_lobby = None;
        true
    }

    fn poll_events(&mut self, cx: &mut GameCx<'_>) {
        let host_events: Vec<NetEvent> = cx
            .state
            .net_host
            .as_ref()
            .map(|h| std::iter::from_fn(|| h.event_rx.try_recv().ok()).collect())
            .unwrap_or_default();
        for evt in &host_events {
            self.process_host_event(cx, evt);
        }
        let client_events: Vec<NetEvent> = cx
            .state
            .net_client
            .as_ref()
            .map(|c| std::iter::from_fn(|| c.event_rx.try_recv().ok()).collect())
            .unwrap_or_default();
        for evt in &client_events {
            self.process_client_event(cx, evt);
        }
    }

    fn process_host_event(&mut self, cx: &mut GameCx<'_>, evt: &NetEvent) {
        match evt {
            NetEvent::ClientHello(id, hello) => {
                self.status = format!("client {id} joined");
                let mut changed = false;
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    let idx = id + 1;
                    if let Some(slot) = lobby.players.get_mut(idx) {
                        slot.name = hello.name.clone();
                        slot.suit_color = hello.suit;
                        slot.ski_color = hello.ski;
                        slot.is_host = false;
                        slot.ready = false;
                        changed = true;
                    }
                }
                if changed {
                    let sys = format!("{} joined", hello.name);
                    self.chat.push_system(sys.clone());
                    let sys_msg = ServerMsg::Chat(ChatMsg {
                        from: String::new(),
                        text: sys,
                    });
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(sys_msg);
                    }
                    let (snap, should_start) = if let Some(ref lobby) = cx.state.pending_lobby {
                        let snap = snapshot_from_lobby(lobby);
                        let all_ready = all_ready(lobby);
                        (snap, all_ready)
                    } else {
                        return;
                    };
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(ServerMsg::Lobby(snap));
                        if should_start {
                            self.pending_start = true;
                        }
                    }
                }
            }
            NetEvent::ClientReady(id, ready) => {
                self.status = format!("client {id} ready {ready}");
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    let idx = id + 1;
                    if let Some(p) = lobby.players.get_mut(idx) {
                        p.ready = *ready;
                    }
                }
                let (snap, should_start) = if let Some(ref lobby) = cx.state.pending_lobby {
                    let snap = snapshot_from_lobby(lobby);
                    let all_ready = all_ready(lobby);
                    (snap, all_ready)
                } else {
                    return;
                };
                if let Some(ref host) = cx.state.net_host {
                    host.broadcast(ServerMsg::Lobby(snap));
                    if should_start {
                        self.pending_start = true;
                    }
                }
            }
            NetEvent::ClientChat(id, text) => {
                if let Some(ref lobby) = cx.state.pending_lobby {
                    let idx = id + 1;
                    let name = &lobby.players[idx].name;
                    self.chat.push_message(name.clone(), text.clone());
                    let msg = ServerMsg::Chat(ChatMsg {
                        from: name.clone(),
                        text: text.clone(),
                    });
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(msg);
                    }
                }
            }
            NetEvent::ClientLeft(id) => {
                self.status = format!("client {id} left");
                let mut name = String::new();
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    let idx = id + 1;
                    if let Some(p) = lobby.players.get_mut(idx) {
                        name = p.name.clone();
                        *p = LobbyPlayer::empty_slot();
                    }
                }
                if !name.is_empty() {
                    let sys = format!("{name} left");
                    self.chat.push_system(sys.clone());
                    let sys_msg = ServerMsg::Chat(ChatMsg {
                        from: String::new(),
                        text: sys,
                    });
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(sys_msg);
                    }
                }
                if let Some(ref lobby) = cx.state.pending_lobby {
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(ServerMsg::Lobby(snapshot_from_lobby(lobby)));
                    }
                }
            }
            NetEvent::Error(e) => self.status = format!("host error: {e}"),
            _ => {}
        }
    }

    fn process_client_event(&mut self, cx: &mut GameCx<'_>, evt: &NetEvent) {
        match evt {
            NetEvent::Connected => self.status = "tcp connected".to_string(),
            NetEvent::ServerMsg(ServerMsg::Chat(msg)) => {
                if msg.from.is_empty() {
                    self.chat.push_system(msg.text.clone());
                } else {
                    self.chat.push_message(msg.from.clone(), msg.text.clone());
                }
            }
            NetEvent::ServerMsg(ServerMsg::Lobby(snap)) => {
                self.status = format!("lobby sync: {} players", snap.players.len());
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    let local_name = lobby.players[lobby.local_idx].name.clone();
                    for player in &mut lobby.players {
                        *player = LobbyPlayer::empty_slot();
                    }
                    for (i, player) in snap.players.iter().enumerate() {
                        if i >= lobby.players.len() {
                            break;
                        }
                        lobby.players[i].name = player.name.clone();
                        lobby.players[i].ready = player.ready;
                        lobby.players[i].is_host = player.is_host;
                        lobby.players[i].suit_color = player.suit;
                        lobby.players[i].ski_color = player.ski;
                    }
                    if let Some((idx, _)) = lobby
                        .players
                        .iter()
                        .enumerate()
                        .find(|(_, p)| !p.is_host && p.name == local_name)
                    {
                        lobby.local_idx = idx;
                    }
                    lobby.hill_idx = snap.hill_idx;
                    lobby.total_legs = snap.total_legs;
                }
            }
            NetEvent::ServerMsg(ServerMsg::JumpRound(round)) => {
                let local_idx = cx
                    .state
                    .pending_lobby
                    .as_ref()
                    .map_or(0, |lobby| lobby.local_idx);
                let total_legs = cx
                    .state
                    .pending_lobby
                    .as_ref()
                    .map_or(20, |lobby| lobby.total_legs);
                let mut runtime = MultiplayerRuntime::new(
                    round.hill_idx,
                    total_legs,
                    round.start_gate,
                    Vec::new(),
                    local_idx,
                );
                runtime.apply_round(round);
                cx.state.mp_jump = Some(runtime);
            }
            NetEvent::ServerMsg(ServerMsg::StandingsUpdate { round, entries }) => {
                if let Some(ref mut mp) = cx.state.mp_jump {
                    mp.sync_standings(*round, entries.clone());
                } else {
                    let (hill_idx, total_legs, local_idx) =
                        cx.state.pending_lobby.as_ref().map_or((0, 20, 0), |lobby| {
                            (lobby.hill_idx, lobby.total_legs, lobby.local_idx)
                        });
                    let mut runtime = MultiplayerRuntime::new(
                        hill_idx,
                        total_legs,
                        15,
                        entries.clone(),
                        local_idx,
                    );
                    runtime.sync_standings(*round, entries.clone());
                    cx.state.mp_jump = Some(runtime);
                }
                cx.state.pending_lobby = None;
            }
            NetEvent::Disconnected => {
                cx.state.net_client = None;
                cx.state.pending_lobby = None;
                self.chat.push_system("server disconnected");
                self.phase = LobbyPhase::Failed;
            }
            NetEvent::Error(e) => {
                self.status = format!("client error: {e}");
                self.phase = LobbyPhase::Failed;
            }
            _ => {}
        }
    }
}

fn all_ready(lobby: &LobbyState) -> bool {
    lobby
        .players
        .iter()
        .filter(|p| !p.is_empty())
        .all(|p| p.ready)
}

fn snapshot_from_lobby(lobby: &LobbyState) -> LobbySnapshot {
    let players: Vec<PlayerInfo> = lobby
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.is_empty())
        .map(|(id, p)| PlayerInfo {
            id,
            name: p.name.clone(),
            ready: p.ready,
            is_host: p.is_host,
            suit: p.suit_color,
            ski: p.ski_color,
        })
        .collect();
    LobbySnapshot {
        players,
        hill_idx: lobby.hill_idx,
        total_legs: lobby.total_legs,
    }
}
