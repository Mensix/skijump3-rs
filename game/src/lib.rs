pub extern crate engine;

pub mod parsers;
pub mod loaders;
pub mod views;

use engine::ui::{Router, RouteTarget, Event, Key};
use loaders::assets::AssetStore;
use parsers::{AssetParser, anim::AnimParser, pcx::PcxParser};
use views::MainMenuView;
use engine::ui::Font;

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";

pub fn run() -> Result<(), String> {
    let mut renderer = engine::video::Renderer::new()?;

    let pcx_data = AssetStore::read(MAIN_PCX).map_err(|e| e.to_string())?;
    let decoded = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;

    let anim_data = AssetStore::read(ANIM_SKI).map_err(|e| e.to_string())?;
    let sprites = AnimParser::parse(&anim_data).map_err(|e| e.to_string())?;

    let mut font = Font::new();
    for (i, sprite) in sprites.iter().enumerate() {
        if i < 67 {
            font.set_glyph(i, sprite.data.clone(), sprite.width, sprite.height, sprite.center_x, sprite.center_y);
        }
    }

    renderer.set_palette(decoded.palette.clone());

    let mut router = Router::new(
        RouteTarget::MainMenu,
        Box::new(MainMenuView::new()),
        vec![
            (RouteTarget::MainMenu, Box::new(|| Box::new(MainMenuView::new()))),
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

        let mut pixels = decoded.pixels.clone();
        {
            let mut ctx = engine::ui::PaintCtx::new(
                &mut pixels,
                &decoded.palette,
                320,
                200,
            );
            let elements = router.current_view().elements();
            for el in &elements {
                el.render(&mut ctx, &font, &[]);
            }
        }
        renderer.blit(&pixels);
        renderer.present()?;
        renderer.wait_frame();
    }

    Ok(())
}