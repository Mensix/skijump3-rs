use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveManager;
use crate::store::GameState;
use engine::oxide::input::UiEvent;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx};

pub struct GameCx<'a> {
    pub state: &'a mut GameState,
    pub save_manager: &'a SaveManager,
    pub layout: &'a MainLayout,
}

pub trait GameScreen {
    fn update(&mut self, _cx: &mut GameCx<'_>) {}
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent);
    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>);
    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}
