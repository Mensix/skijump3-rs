use engine::oxide::input::UiEvent;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuAction, PixelMenu};
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx};

use crate::gfx::theme::{
    BG_DARK, BG_PURPLE, BG_RED, BLACK, FILL_DARK, FILL_GOLD, FILL_GRAY, FILL_TEAL, FONT_BODY,
    FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
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
                let local = LobbyPlayer::from_profile(0, p, true);
                let lobby = LobbyState::new(true, local);
                cx.state.pending_lobby = Some(lobby);
                nav.navigate(RouteTarget::MultiplayerLobby);
            }
            Some(MenuAction::Item(1)) => {
                let p = &cx.state.profiles.profiles[0];
                let local = LobbyPlayer::from_profile(0, p, false);
                let lobby = LobbyState::new(false, local);
                cx.state.pending_lobby = Some(lobby);
                nav.navigate(RouteTarget::MultiplayerLobby);
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

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_DARK);
        paint.pattern_fill((0, 0, 320, 39), BG_PURPLE);
        paint.fill((0, 39, 320, 1), FILL_GOLD);

        paint.center_text((160, 10), FONT_GOLD, "MULTIPLAYER");
        paint.center_text(
            (160, 22),
            FONT_GRAY,
            "host a hill room or join another jumper",
        );

        paint_panel(paint, 14, 49, 292, 120, "NETPLAY CONTROL");
        let selected = self.menu.selected();
        paint_menu_card(
            paint,
            24,
            62,
            272,
            30,
            selected == 0,
            "1",
            "HOST GAME",
            "Create a private room and choose the hill.",
        );
        paint_menu_card(
            paint,
            24,
            100,
            272,
            30,
            selected == 1,
            "2",
            "JOIN GAME",
            "Connect to a host and wait in the lobby.",
        );
        paint_menu_card(
            paint,
            24,
            138,
            272,
            20,
            selected == 2,
            "3",
            "BACK",
            "Return to main menu.",
        );

        let local = cx
            .state
            .profiles
            .profiles
            .first()
            .map(|p| p.name.as_str())
            .unwrap_or("Player");
        paint.fill((14, 174, 292, 15), FILL_DARK);
        paint.stroke((14, 174, 292, 15), FILL_GRAY);
        paint.text((20, 178), FONT_GRAY, "LOCAL JUMPER");
        paint.text((106, 178), FONT_GOLD, local);
        paint.right_text((300, 178), FONT_TEAL, "Arrows / Enter / Esc");
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

fn paint_panel(paint: &mut PaintCx<'_>, x: i32, y: i32, w: i32, h: i32, title: &str) {
    paint.fill((x, y, w, h), BLACK);
    paint.pattern_fill((x + 1, y + 1, w - 2, h - 2), BG_RED);
    paint.stroke((x, y, w, h), FILL_GOLD);
    paint.fill(
        (x + 8, y - 4, paint.string_width(title) as i32 + 8, 9),
        BLACK,
    );
    paint.text((x + 12, y - 3), FONT_GOLD, title);
}

#[allow(clippy::too_many_arguments)]
fn paint_menu_card(
    paint: &mut PaintCx<'_>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    selected: bool,
    key: &str,
    title: &str,
    desc: &str,
) {
    let border = if selected { FONT_GOLD } else { FILL_GRAY };
    let fill = if selected { FILL_GOLD } else { FILL_DARK };
    paint.fill((x, y, w, h), fill);
    paint.stroke((x, y, w, h), border);
    paint.fill((x + 5, y + 5, 18, h - 10), BLACK);
    paint.stroke((x + 5, y + 5, 18, h - 10), border);
    paint.center_text(
        (x + 14, y + 10),
        if selected { FONT_GOLD } else { FONT_GRAY },
        key,
    );
    paint.text(
        (x + 31, y + 5),
        if selected { FONT_GOLD } else { FONT_BODY },
        title,
    );
    paint.text(
        (x + 31, y + 17),
        if selected { FONT_BODY } else { FONT_GRAY },
        desc,
    );
    if selected {
        paint.fill((x + w - 18, y + 6, 10, 2), FILL_TEAL);
        paint.fill((x + w - 14, y + 10, 10, 2), FILL_TEAL);
        paint.fill((x + w - 10, y + 14, 10, 2), FILL_TEAL);
    }
}
