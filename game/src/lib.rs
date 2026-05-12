pub extern crate engine;

pub mod parsers;
pub mod loaders;
pub mod components;
pub mod route;
pub mod views;

use engine::ui::{Router, Event, Key};
use engine::sprite::SpriteData;
use loaders::assets::AssetStore;
use parsers::{AssetParser, anim::AnimParser, langbase::LangBaseParser, pcx::PcxParser};
use views::MainMenuView;
use engine::ui::Font;
use route::RouteTarget;

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";
const LANGBASE_SKI: &str = "LANGBASE.SKI";
const VERSION: &str = "3.12";

const STANDARD_UI_PALETTE: [[u8; 3]; 36] = [
    [53, 17, 53], [63,  0,  0], [43, 12, 43], [63,  0,  0],
    [49, 45,  0], [34, 31,  0], [63,  0,  0], [56, 54, 54],
    [63, 63, 21], [54, 52, 10], [42, 42, 42], [42, 20, 10],
    [21, 21, 21], [57, 45, 38], [63,  0,  0], [63, 63, 32],
    [40, 40, 41], [48, 48, 49], [55, 55, 56], [63, 63, 63],
    [56, 13, 13], [13, 53, 13], [23, 23, 63], [63, 23, 23],
    [63, 63, 63], [44, 44, 44], [ 0,  0,  0], [18, 13, 34],
    [34, 13, 18], [20, 20, 20], [63, 57,  9], [ 9, 57, 63],
    [23, 16, 43], [43, 16, 23], [26, 26, 26], [52, 47,  0],
];

fn apply_standard_ui_palette(palette: &mut engine::palette::Palette) {
    for (i, &rgb) in STANDARD_UI_PALETTE.iter().enumerate() {
        palette.set(216 + i, rgb);
    }
}

pub fn run() -> Result<(), String> {
    let mut renderer = engine::video::Renderer::new()?;

    let pcx_data = AssetStore::read(MAIN_PCX).map_err(|e| e.to_string())?;
    let decoded = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;

    let anim_data = AssetStore::read(ANIM_SKI).map_err(|e| e.to_string())?;
    let sprites: Vec<SpriteData> = AnimParser::parse(&anim_data).map_err(|e| e.to_string())?;

    let langbase_data = AssetStore::read(LANGBASE_SKI).map_err(|e| e.to_string())?;
    let langbase = LangBaseParser::parse(&langbase_data).map_err(|e| e.to_string())?;

    let mut font = Font::new();
    for (i, sprite) in sprites.iter().enumerate() {
        if i < 67 {
            font.set_glyph(i, sprite.data.clone(), sprite.width, sprite.height, sprite.center_x, sprite.center_y);
        }
    }

    let mut palette = decoded.palette.clone();
    apply_standard_ui_palette(&mut palette);
    renderer.set_palette(palette);

    let lb_main = langbase.clone();
    let lb_play1 = langbase.clone();
    let lb_play2 = langbase.clone();
    let lb_quit = langbase.clone();

    let mut router: Router<RouteTarget> = Router::new(
        RouteTarget::MainMenu,
        Box::new(MainMenuView::new(langbase, VERSION.to_string())),
        vec![
            (RouteTarget::MainMenu, Box::new(move || Box::new(MainMenuView::new(lb_main.clone(), VERSION.to_string())))),
            (RouteTarget::Play(1), Box::new(move || Box::new(MainMenuView::new(lb_play1.clone(), VERSION.to_string())))),
            (RouteTarget::Play(2), Box::new(move || Box::new(MainMenuView::new(lb_play2.clone(), VERSION.to_string())))),
            (RouteTarget::Quit, Box::new(move || Box::new(MainMenuView::new(lb_quit.clone(), VERSION.to_string())))),
        ],
    );

    while renderer.running() {
        if router.current_route() == Some(&RouteTarget::Quit) {
            break;
        }
        renderer.poll_input();

        if let Some(key) = renderer.last_key() {
            let event = match key {
                sdl2::keyboard::Keycode::Up => Event::Keyboard(Key::Up),
                sdl2::keyboard::Keycode::Down => Event::Keyboard(Key::Down),
                sdl2::keyboard::Keycode::Return => Event::Keyboard(Key::Enter),
                sdl2::keyboard::Keycode::Escape => Event::Keyboard(Key::Escape),
                sdl2::keyboard::Keycode::Num0 => Event::Keyboard(Key::Char('0')),
                sdl2::keyboard::Keycode::Num1 => Event::Keyboard(Key::Char('1')),
                sdl2::keyboard::Keycode::Num2 => Event::Keyboard(Key::Char('2')),
                sdl2::keyboard::Keycode::Num3 => Event::Keyboard(Key::Char('3')),
                sdl2::keyboard::Keycode::Num4 => Event::Keyboard(Key::Char('4')),
                sdl2::keyboard::Keycode::Num5 => Event::Keyboard(Key::Char('5')),
                sdl2::keyboard::Keycode::Num6 => Event::Keyboard(Key::Char('6')),
                sdl2::keyboard::Keycode::Num7 => Event::Keyboard(Key::Char('7')),
                sdl2::keyboard::Keycode::Num8 => Event::Keyboard(Key::Char('8')),
                sdl2::keyboard::Keycode::Num9 => Event::Keyboard(Key::Char('9')),
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
                el.render(&mut ctx, &font, &sprites);
            }
        }
        renderer.blit(&pixels);
        renderer.present()?;
        renderer.wait_frame();
    }

    Ok(())
}
