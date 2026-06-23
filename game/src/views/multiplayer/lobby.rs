use std::net::SocketAddr;
use std::thread::JoinHandle;
use std::time::Duration;

use engine::oxide::input::{Key, UiEvent};
use engine::oxide::paint::PaintCx;
use engine::oxide::{ScreenBackground, ScreenEventCx};
use net::client::ClientHandle;
use net::discovery;
use net::host::HostHandle;
use net::protocol::{self, ClientMsg, Hello, PlayerInfo, LobbySnapshot, NetEvent, ServerMsg};

use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::views::multiplayer::state::{LobbyPlayer, LobbyState};

pub struct MultiplayerLobbyView {
    phase: LobbyPhase,
    discovery_handle: Option<JoinHandle<Option<SocketAddr>>>,
    init_done: bool,
}

enum LobbyPhase {
    Discovering,
    Connected,
}

impl MultiplayerLobbyView {
    pub fn new() -> Self {
        Self {
            phase: LobbyPhase::Discovering,
            discovery_handle: None,
            init_done: false,
        }
    }
}

impl GameScreen for MultiplayerLobbyView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        if !self.init_done {
            self.init_done = true;
            match HostHandle::start() {
                Ok(host) => {
                    cx.state.net_host = Some(host);
                    let p = &cx.state.profiles.profiles[0];
                    let local = LobbyPlayer::from_profile(0, p, true);
                    cx.state.pending_lobby = Some(LobbyState::new(true, local));
                    self.phase = LobbyPhase::Connected;
                    return;
                }
                Err(_) => self.start_discovery(),
            }
        }
        if let LobbyPhase::Discovering = self.phase {
            self.tick_discovery(cx);
        }
        if let LobbyPhase::Connected = self.phase {
            self.poll_events(cx);
        }
    }
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::KeyDown(Key::Escape) => {
                cx.state.net_host.take();
                cx.state.net_client.take();
                nav.back();
            }
            UiEvent::KeyDown(Key::F1) => {
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    lobby.players[lobby.local_idx].ready = !lobby.players[lobby.local_idx].ready;
                    if let Some(ref host) = cx.state.net_host {
                        let snapshot = snapshot_from_lobby(lobby);
                        host.broadcast(ServerMsg::Lobby(snapshot));
                    }
                    if let Some(ref client) = cx.state.net_client {
                        let ready = lobby.players[lobby.local_idx].ready;
                        client.send(ClientMsg::Ready(ready));
                    }
                }
                nav.consume();
            }
            UiEvent::KeyDown(Key::F2) => {
                nav.navigate(RouteTarget::CompetitionJump);
                nav.consume();
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        paint.pattern_fill((0, 20, 320, 180), BG_PURPLE);
        paint.sprite(sprites::Sprite::Logo as u16, (5, 2));

        match self.phase {
            LobbyPhase::Discovering => {
                paint.center_text((160, 90), FONT_GOLD, "Scanning WiFi...");
                paint.center_text((160, 102), FONT_GRAY, "looking for existing rooms");
            }
            LobbyPhase::Connected => {
                let mode = if cx.state.net_host.is_some() {
                    "ROOM"
                } else if cx.state.net_client.is_some() {
                    "JOINED"
                } else {
                    "OFFLINE"
                };
                paint.text((30, 6), FONT_BODY, format!("MULTIPLAYER {mode}"));
                paint.right_text((316, 5), FONT_GRAY, "F1 ready  F2 start)");
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
                                if is_local { FONT_GOLD } else { FONT_TEAL }
                            } else {
                                FONT_GRAY
                            },
                            if p.ready { "RDY" } else { "WAIT" },
                        );
                    }
                }
            }
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

impl MultiplayerLobbyView {
    fn start_discovery(&mut self) {
        let h = std::thread::spawn(|| {
            match discovery::discover(Duration::from_secs(2)) {
                Ok(Some(addr)) => Some(addr),
                _ => None,
            }
        });
        self.discovery_handle = Some(h);
    }

    fn tick_discovery(&mut self, cx: &mut GameCx<'_>) {
        let Some(handle) = self.discovery_handle.take() else { return };
        if !handle.is_finished() {
            self.discovery_handle = Some(handle);
            return;
        }
        if let Ok(Some(addr)) = handle.join() {
            let p = &cx.state.profiles.profiles[0];
            let hello = Hello {
                name: p.name.clone(),
                suit: p.suit_color,
                ski: p.ski_color,
            };
            if let Ok(client) = ClientHandle::connect(addr, hello) {
                cx.state.net_client = Some(client);
                let local = LobbyPlayer::from_profile(0, p, false);
                cx.state.pending_lobby = Some(LobbyState::new(false, local));
                self.phase = LobbyPhase::Connected;
                return;
            }
        }
        self.discovery_handle = None;
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

    fn process_host_event(&self, cx: &mut GameCx<'_>, evt: &NetEvent) {
        match evt {
            NetEvent::ClientHello(_id, hello) => {
                let mut changed = false;
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    if let Some(slot) = lobby.players.iter_mut().find(|p| p.is_empty()) {
                        slot.name = hello.name.clone();
                        slot.suit_color = hello.suit;
                        slot.ski_color = hello.ski;
                        slot.is_host = false;
                        slot.ready = false;
                        changed = true;
                    }
                }
                if changed {
                    let (snap, should_start) = if let Some(ref lobby) = cx.state.pending_lobby {
                        let snap = snapshot_from_lobby(lobby);
                        let should = lobby.players.iter().filter(|p| !p.is_empty()).count() >= 2
                            && lobby.players.iter().filter(|p| !p.is_empty()).all(|p| p.ready);
                        (snap, should)
                    } else {
                        return;
                    };
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(ServerMsg::Lobby(snap));
                        if should_start {
                            let msg = ServerMsg::Start(protocol::Start {
                                hill_idx: 0,
                                seed: 42,
                            });
                            host.broadcast(msg);
                            cx.state.pending_lobby = None;
                        }
                    }
                }
            }
            NetEvent::ClientReady(_id, ready) => {
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    if let Some(p) = lobby.players.iter_mut().find(|p| !p.is_empty() && !p.is_host) {
                        p.ready = *ready;
                    }
                }
                let (snap, should_start) = if let Some(ref lobby) = cx.state.pending_lobby {
                    let snap = snapshot_from_lobby(lobby);
                    let should = lobby.players.iter().filter(|p| !p.is_empty()).count() >= 2
                        && lobby.players.iter().filter(|p| !p.is_empty()).all(|p| p.ready);
                    (snap, should)
                } else {
                    return;
                };
                if let Some(ref host) = cx.state.net_host {
                    host.broadcast(ServerMsg::Lobby(snap));
                    if should_start {
                        let msg = ServerMsg::Start(protocol::Start {
                            hill_idx: 0,
                            seed: 42,
                        });
                        host.broadcast(msg);
                        cx.state.pending_lobby = None;
                    }
                }
            }
            NetEvent::ClientLeft(_id) => {
                if let Some(ref mut lobby) = cx.state.pending_lobby {
                    if let Some(p) = lobby.players.iter_mut().find(|p| !p.is_empty() && !p.is_host) {
                        *p = LobbyPlayer::empty_slot();
                    }
                }
                if let Some(ref lobby) = cx.state.pending_lobby {
                    if let Some(ref host) = cx.state.net_host {
                        host.broadcast(ServerMsg::Lobby(snapshot_from_lobby(lobby)));
                    }
                }
            }
            NetEvent::Error(_e) => {
                cx.state.pending_lobby = None;
            }
            _ => {}
        }
    }

    fn process_client_event(&self, cx: &mut GameCx<'_>, evt: &NetEvent) {
        match evt {
            NetEvent::ServerMsg(ServerMsg::Lobby(snap)) => {
                if let Some(ref mut lobby) = cx.state.pending_lobby {
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
                }
            }
            NetEvent::ServerMsg(ServerMsg::Start(_)) => {
                cx.state.pending_lobby = None;
            }
            NetEvent::Disconnected => {
                cx.state.net_client = None;
                cx.state.pending_lobby = None;
            }
            _ => {}
        }
    }
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
    }
}
