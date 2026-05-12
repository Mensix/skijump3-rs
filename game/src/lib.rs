pub extern crate engine;

pub mod parsers;
pub mod loaders;
pub mod components;
pub mod data;
pub mod route;
pub mod store;
pub mod views;

use std::sync::Arc;
use engine::ui::Router;
use engine::input::Input;
use engine::sprite::SpriteData;
use engine::consts::{WIDTH, HEIGHT, FONT_GLYPH_COUNT};
use loaders::assets::AssetStore;
use parsers::{AssetParser, anim::AnimParser, langbase::{LangBase, LangBaseParser}, pcx::PcxParser};
use views::{MainMenuView, JumpMenuView, ProfilesView};
use crate::components::layout::MainLayout;
use crate::store::{Store, StoreRef};
use engine::ui::Font;
use engine::palette::Palette;
use route::RouteTarget;

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";
const LANGBASE_SKI: &str = "LANGBASE.SKI";
const VERSION: &str = "3.12";
const UI_PALETTE_BASE: usize = 216;

const STANDARD_UI_PALETTE: [[u8; 3]; 40] = [
    [53, 17, 53], [63,  0,  0], [43, 12, 43], [63,  0,  0],
    [49, 45,  0], [34, 31,  0], [63,  0,  0], [56, 54, 54],
    [63, 63, 21], [54, 52, 10], [42, 42, 42], [42, 20, 10],
    [21, 21, 21], [57, 45, 38], [63,  0,  0], [63, 63, 32],
    [40, 40, 41], [48, 48, 49], [55, 55, 56], [63, 63, 63],
    [56, 13, 13], [13, 53, 13], [23, 23, 63], [63, 23, 23],
    [63, 63, 63], [44, 44, 44], [ 0,  0,  0], [18, 13, 34],
    [34, 13, 18], [20, 20, 20], [63, 57,  9], [ 9, 57, 63],
    [23, 16, 43], [43, 16, 23], [26, 26, 26], [52, 47,  0],
    [ 0, 47, 52], [51, 51, 51], [38, 38, 38], [63, 63, 63],
];

fn apply_standard_ui_palette(palette: &mut Palette) {
    for (i, &rgb) in STANDARD_UI_PALETTE.iter().enumerate() {
        palette.set(UI_PALETTE_BASE + i, rgb);
    }
    // MuutaLogo(0): blue logo colors at 253-254
    palette.set(253, [46, 46, 63]);
    palette.set(254, [32, 32, 63]);
}

fn load_font(sprites: &[SpriteData]) -> Font {
    let mut font = Font::new();
    for (i, sprite) in sprites.iter().enumerate() {
        if i < FONT_GLYPH_COUNT {
            font.set_glyph(i, sprite.data.clone(), sprite.width, sprite.height, sprite.center_x, sprite.center_y);
        }
    }
    font
}

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: engine::video::Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
    sprites: Vec<SpriteData>,
    framebuffer: Vec<u8>,
    palette: Palette,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let (sdl, mut renderer, input) = Self::init_sdl()?;
        let (pixels, pcx_palette, sprites, langbase) = Self::load_assets()?;

        let font = load_font(&sprites);
        let framebuffer = vec![0u8; (WIDTH * HEIGHT) as usize];

        let mut palette = pcx_palette;
        apply_standard_ui_palette(&mut palette);
        renderer.set_palette(palette.clone());

        let store: StoreRef = std::rc::Rc::new(std::cell::RefCell::new(Store::new(font.clone())));
        let router = Self::create_router(Arc::clone(&langbase), pixels.clone(), store);

        Ok(Self {
            _sdl: sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            framebuffer,
            palette,
        })
    }

    fn init_sdl() -> Result<(sdl2::Sdl, engine::video::Renderer, Input), String> {
        let sdl = sdl2::init().map_err(|e| e.to_string())?;
        let renderer = engine::video::Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;
        Ok((sdl, renderer, input))
    }

    fn load_assets() -> Result<(Vec<u8>, Palette, Vec<SpriteData>, Arc<LangBase>), String> {
        let pcx_data = AssetStore::read(MAIN_PCX).map_err(|e| e.to_string())?;
        let decoded = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;

        let anim_data = AssetStore::read(ANIM_SKI).map_err(|e| e.to_string())?;
        let sprites: Vec<SpriteData> = AnimParser::parse(&anim_data).map_err(|e| e.to_string())?;

        let langbase_data = AssetStore::read(LANGBASE_SKI).map_err(|e| e.to_string())?;
        let langbase = Arc::new(LangBaseParser::parse(&langbase_data).map_err(|e| e.to_string())?);

        Ok((decoded.pixels, decoded.palette, sprites, langbase))
    }

    fn create_router(langbase: Arc<LangBase>, background: Vec<u8>, store: StoreRef) -> Router<RouteTarget> {
        let layout = MainLayout::new(Arc::clone(&langbase), VERSION.to_string(), background);
        Router::new(
            RouteTarget::ProfilesList,
            Box::new(ProfilesView::new(store.clone())),
            vec![
                (RouteTarget::MainMenu, {
                    let l = layout.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone())))
                }),
                (RouteTarget::JumpMenu, {
                    let l = layout.clone();
                    Box::new(move || Box::new(JumpMenuView::new(l.clone())))
                }),
                (RouteTarget::ProfilesList, {
                    let s = store.clone();
                    Box::new(move || Box::new(ProfilesView::new(s.clone())))
                }),
                (RouteTarget::OptionsMenu, {
                    let l = layout.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone())))
                }),
                (RouteTarget::Quit, {
                    let l = layout.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone())))
                }),
            ],
        )
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.input.running() {
            if self.router.current_route() == Some(&RouteTarget::Quit) {
                break;
            }
            self.handle_input();
            self.render_frame()?;
        }
        Ok(())
    }

    fn handle_input(&mut self) {
        for event in self.input.drain_events() {
            self.router.handle_event(&event);
        }
    }

    fn render_frame(&mut self) -> Result<(), String> {
        self.framebuffer.fill(0);
        let mut ctx = engine::ui::PaintCtx::new(&mut self.framebuffer, &self.palette, WIDTH, HEIGHT);
        let elements = self.router.current_view().elements();
        for el in &elements {
            el.render(&mut ctx, &self.font, &self.sprites);
        }
        self.renderer.blit(&self.framebuffer);
        self.renderer.present()?;
        self.renderer.wait_frame();
        Ok(())
    }
}

pub fn run() {
    let mut game = match Game::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = game.run() {
        eprintln!("{}", e);
    }
}
