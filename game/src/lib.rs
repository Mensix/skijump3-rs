pub extern crate engine;

pub mod parsers;
pub mod loaders;
pub mod views;

use engine::ui::{Router, RouteTarget, Event, Key};
use loaders::main_menu::MainMenuAssets;
use views::{MainMenuView, RaceMenuView};

pub fn run() -> Result<(), String> {
    let mut renderer = engine::video::Renderer::new()?;
    let assets = MainMenuAssets::load()?;
    renderer.set_palette(assets.background.palette.clone());

    let mut router = Router::new(
        RouteTarget::MainMenu,
        Box::new(MainMenuView::new()),
        vec![
            (RouteTarget::MainMenu, Box::new(|| Box::new(MainMenuView::new()))),
            (RouteTarget::RaceMenu, Box::new(|| Box::new(RaceMenuView::new()))),
            (RouteTarget::Play(0), Box::new(|| Box::new(MainMenuView::new()))),
            (RouteTarget::Play(1), Box::new(|| Box::new(MainMenuView::new()))),
            (RouteTarget::Play(2), Box::new(|| Box::new(MainMenuView::new()))),
        ],
    );

    while renderer.running() {
        renderer.poll_input();

        if let Some(key) = renderer.last_key() {
            let event = match key {
                sdl2::keyboard::Keycode::Up => Event::Keyboard(Key::Up),
                sdl2::keyboard::Keycode::Down => Event::Keyboard(Key::Down),
                sdl2::keyboard::Keycode::Return => Event::Keyboard(Key::Enter),
                sdl2::keyboard::Keycode::Escape => Event::Keyboard(Key::Escape),
                _ => continue,
            };
            router.handle_event(event);
        }

        let mut pixels = assets.background.pixels.clone();
        {
            let mut ctx = engine::ui::PaintCtx::new(
                &mut pixels,
                &assets.background.palette,
                320,
                200,
            );
            router.paint(&mut ctx, &[], &assets.font);
        }
        renderer.blit(&pixels);
        renderer.present()?;
        renderer.wait_frame();
    }

    Ok(())
}