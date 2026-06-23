use engine::oxide::input::{Key, UiEvent};
use engine::oxide::paint::PaintCx;
use engine::oxide::widget::EventCx;
use engine::oxide::ScreenEventCx;
use engine::oxide::{Blinker, TextEditState};

use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GOLD, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::views::multiplayer::state::{LobbyPlayer, LobbyState, MAX_PLAYERS};

use super::chat;

pub struct MultiplayerLobbyView {
    pub lobby: LobbyState,
    input: TextEditState,
    blinker: Blinker,
}

impl MultiplayerLobbyView {
    pub fn new(lobby: LobbyState) -> Self {
        Self {
            lobby,
            input: TextEditState::new(String::new(), 60),
            blinker: Blinker::new(),
        }
    }
}

impl GameScreen for MultiplayerLobbyView {
    fn event(
        &mut self,
        _cx: &mut GameCx<'_>,
        nav: &mut ScreenEventCx<RouteTarget>,
        event: UiEvent,
    ) {
        let mut ecx = EventCx::default();

        match event {
            UiEvent::KeyDown(Key::Escape) => {
                nav.back();
                return;
            }
            UiEvent::KeyDown(Key::F1) => {
                self.lobby.toggle_ready();
                nav.consume();
                return;
            }
            UiEvent::KeyDown(Key::F2) => {
                if self.lobby.is_host && self.lobby.all_ready() {
                    nav.navigate(RouteTarget::CompetitionJump);
                }
                nav.consume();
                return;
            }
            _ => {}
        }

        let mut on_send = |_text: String| {};
        chat::handle_chat_event(
            &mut self.lobby.chat,
            &mut self.input,
            event,
            &mut ecx,
            &mut on_send,
        );

        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        paint.pattern_fill((0, 20, 320, 135), BG_PURPLE);
        paint.sprite(sprites::Sprite::Logo as u16, (5, 2));

        let mode = if self.lobby.is_host {
            "HOST ROOM"
        } else {
            "CLIENT ROOM"
        };
        let connected = self.lobby.connected_count();

        paint.text((30, 6), FONT_BODY, format!("SJ3 NETPLAY - {mode}"));
        paint.right_text((316, 5), FONT_GRAY, format!("{connected}/{MAX_PLAYERS}"));
        paint.right_text((316, 14), FONT_GRAY, "Esc leave  F1 ready  F2 start");

        paint_lobby_roster(paint, &self.lobby);

        paint.fill((0, 156, 320, 1), FILL_GOLD);
        chat::paint_chat(paint, &self.lobby.chat, self.input.buffer(), &self.blinker);
    }

    fn background(&self) -> engine::oxide::ScreenBackground {
        engine::oxide::ScreenBackground::NoneBlack
    }
}

fn paint_lobby_roster(paint: &mut PaintCx<'_>, lobby: &LobbyState) {
    let mut display_idx = 1;
    for (vec_idx, player) in lobby.players.iter().enumerate().take(MAX_PLAYERS) {
        if player.is_empty() {
            continue;
        }
        let y = 23 + (display_idx - 1) as i32 * 8;
        paint_lobby_player_row(paint, player, display_idx, y, vec_idx == lobby.local_idx);
        display_idx += 1;
    }
}

fn paint_lobby_player_row(
    paint: &mut PaintCx<'_>,
    player: &LobbyPlayer,
    display_idx: usize,
    y: i32,
    is_local: bool,
) {
    paint.right_text((24, y), FONT_GOLD, format!("{}.", display_idx));

    let name = truncate_lobby_name(&player.name);
    let name_color = if is_local { FONT_BODY } else { FONT_GRAY };
    paint.text((32, y), name_color, &name);

    let status = if player.is_host && player.ready {
        "H RDY"
    } else if player.is_host {
        "HOST"
    } else if player.ready {
        "READY"
    } else {
        "WAIT"
    };
    let status_color = if is_local {
        FONT_BODY
    } else if player.ready {
        FONT_TEAL
    } else {
        FONT_GRAY
    };
    paint.right_text((184, y), status_color, status);
}

fn truncate_lobby_name(name: &str) -> String {
    const MAX: usize = 22;
    if name.chars().count() <= MAX {
        name.to_string()
    } else {
        name.chars().take(MAX - 1).collect::<String>() + "~"
    }
}
