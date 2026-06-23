use net::client::ClientHandle;
use net::host::HostHandle;
use net::protocol::Hello;

use engine::oxide::input::UiEvent;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuAction, PixelMenu};
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx};

use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::views::multiplayer::state::{LobbyPlayer, LobbyState};

pub struct MultiplayerMenuView {
    menu: PixelMenu,
}

impl MultiplayerMenuView {
    pub fn new() -> Self {
        use engine::oxide::widgets::menu::MenuItem;
        let items = vec![
            MenuItem::new(0, ""),
            MenuItem::new(1, ""),
            MenuItem::new(2, ""),
        ];
        let menu = PixelMenu::new(18, 58, 284, 38, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        Self { menu }
    }
}

impl GameScreen for MultiplayerMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = EventCx::default();
        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(0)) => {
                let p = &cx.state.profiles.profiles[0];
                let _hello = Hello {
                    name: p.name.clone(),
                    suit: p.suit_color,
                    ski: p.ski_color,
                };
                let local = LobbyPlayer::from_profile(0, p, true);
                let lobby = LobbyState::new(true, local);
                if let Ok(handle) = HostHandle::start() {
                    cx.state.net_host = Some(handle);
                    cx.state.pending_lobby = Some(lobby);
                    nav.navigate(RouteTarget::MultiplayerLobby);
                }
            }
            Some(MenuAction::Item(1)) => {
                let p = &cx.state.profiles.profiles[0];
                let hello = Hello {
                    name: p.name.clone(),
                    suit: p.suit_color,
                    ski: p.ski_color,
                };
                let local = LobbyPlayer::from_profile(0, p, false);
                let lobby = LobbyState::new(false, local);
                let joined = std::thread::spawn(move || {
                    let timeout = std::time::Duration::from_secs(5);
                    if let Ok(Some(addr)) = net::discovery::discover(timeout) {
                        if let Ok(client) = ClientHandle::connect(addr, hello) {
                            return Some(client);
                        }
                    }
                    None
                });
                if let Ok(Some(client)) = joined.join() {
                    cx.state.net_client = Some(client);
                    cx.state.pending_lobby = Some(lobby);
                    nav.navigate(RouteTarget::MultiplayerLobby);
                }
            }
            Some(MenuAction::Item(2)) | Some(MenuAction::Trailing) => {
                nav.back();
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        paint.center_text((160, 50), FONT_GOLD, "MULTIPLAYER");

        for (i, label) in ["1 - HOST GAME", "2 - JOIN GAME", "3 - BACK"]
            .iter()
            .enumerate()
        {
            paint.text((100, 70 + i as i32 * 14), FONT_BODY, *label);
        }
        let sel = self.menu.selected();
        paint.stroke((94, 66 + sel as i32 * 14, 126, 13), FONT_GRAY);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}
