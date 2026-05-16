pub extern crate engine;

pub mod components;
pub mod data;
pub mod jump;
pub mod loaders;
pub mod palette_consts;
pub mod parsers;
pub mod pascal_random;
pub mod route;
pub mod snow;
pub mod store;
pub mod utils;
pub mod views;
pub mod wind;

use crate::components::layout::MainLayout;
use crate::data::records::{HillCatalog, RecordStore};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::consts::{FONT_GLYPH_COUNT, HEIGHT, WIDTH};
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::Font;
use engine::ui::Router;
use loaders::assets::AssetStore;
use parsers::{
    anim::AnimParser,
    hills::HillBaseParser,
    langbase::{LangBase, LangBaseParser},
    names::NamesParser,
    pcx::PcxParser,
    records::RecordsParser,
    AssetParser,
};
use route::RouteTarget;
use std::rc::Rc;
use views::{
    CompetitionJumpView, HallOfFameView, HillRecordsView, JumpMenuView, JumpView, MainMenuView,
    PracticeView, ProfilesView, ReplayBrowserView, ReplayView, WelcomeScreenView,
};

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";
const LANGBASE_SKI: &str = "LANGBASE.SKI";
const HILLBASE_SKI: &str = "HILLBASE.SKI";
const HISCORE_SKI: &str = "HISCORE.SKI";
const NAMES_FILES: &[&str] = &["NAMES0.SKI", "NAMES1.SKI", "NAMES2.SKI"];
const VERSION: &str = "3.12";
const UI_PALETTE_BASE: usize = 216;

const STANDARD_UI_PALETTE: [[u8; 3]; 40] = [
    [53, 17, 53],
    [63, 0, 0],
    [43, 12, 43],
    [63, 0, 0],
    [49, 45, 0],
    [34, 31, 0],
    [63, 0, 0],
    [56, 54, 54],
    [63, 63, 21],
    [54, 52, 10],
    [42, 42, 42],
    [42, 20, 10],
    [21, 21, 21],
    [57, 45, 38],
    [63, 0, 0],
    [63, 63, 32],
    [40, 40, 41],
    [48, 48, 49],
    [55, 55, 56],
    [63, 63, 63],
    [56, 13, 13],
    [13, 53, 13],
    [23, 23, 63],
    [63, 23, 23],
    [63, 63, 63],
    [44, 44, 44],
    [0, 0, 0],
    [18, 13, 34],
    [34, 13, 18],
    [20, 20, 20],
    [63, 57, 9],
    [9, 57, 63],
    [23, 16, 43],
    [43, 16, 23],
    [26, 26, 26],
    [52, 47, 0],
    [0, 47, 52],
    [51, 51, 51],
    [38, 38, 38],
    [63, 63, 63],
];

fn apply_standard_ui_palette(palette: &mut Palette) {
    for (i, &rgb) in STANDARD_UI_PALETTE.iter().enumerate() {
        palette.set(UI_PALETTE_BASE + i, rgb);
    }
}

fn load_font(sprites: &[SpriteData]) -> Font {
    let mut font = Font::new();
    for (i, sprite) in sprites.iter().enumerate() {
        if i < FONT_GLYPH_COUNT {
            font.set_glyph(
                i,
                sprite.data.clone(),
                sprite.width,
                sprite.height,
                sprite.center_x,
                sprite.center_y,
            );
        }
    }
    font
}

fn load_player_names(assets: &AssetStore) -> Vec<String> {
    let mut all_names = Vec::new();
    for &filename in NAMES_FILES {
        if let Ok(data) = assets.read(filename) {
            if let Ok(names) = NamesParser::parse(&data) {
                all_names.extend(names);
            }
        }
    }
    all_names
}

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: engine::video::Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
    sprites: Vec<SpriteData>,
    framebuffer: Vec<u8>,
    base_palette: Palette,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let (sdl, mut renderer, input) = Self::init_sdl()?;
        let assets = AssetStore::new("game/assets");
        let (pixels, pcx_palette, sprites, langbase) = Self::load_assets(&assets)?;

        let font = load_font(&sprites);
        let framebuffer = vec![0u8; (WIDTH * HEIGHT) as usize];

        let mut base_palette = pcx_palette;
        apply_standard_ui_palette(&mut base_palette);
        renderer.set_palette(base_palette.clone());

        let player_names = load_player_names(&assets);
        let hills = Self::load_hills(&assets)?;
        let records = Self::load_records(&assets)?;
        let resources: ResourcesRef =
            Rc::new(Resources::new(font.clone(), langbase, player_names, hills, assets));
        let store: StoreRef = Rc::new(Store::new(records));
        let router = Self::create_router(resources, pixels, store);

        Ok(Self {
            _sdl: sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            framebuffer,
            base_palette,
        })
    }

    fn init_sdl() -> Result<(sdl2::Sdl, engine::video::Renderer, Input), String> {
        let sdl = sdl2::init()?;
        let renderer = engine::video::Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;
        Ok((sdl, renderer, input))
    }

    #[allow(clippy::type_complexity)]
    fn load_assets(assets: &AssetStore) -> Result<(Vec<u8>, Palette, Vec<SpriteData>, Rc<LangBase>), String> {
        let pcx_data = assets.read(MAIN_PCX).map_err(|e| e.to_string())?;
        let decoded = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;

        let anim_data = assets.read(ANIM_SKI).map_err(|e| e.to_string())?;
        let sprites: Vec<SpriteData> = AnimParser::parse(&anim_data).map_err(|e| e.to_string())?;

        let langbase_data = assets.read(LANGBASE_SKI).map_err(|e| e.to_string())?;
        let langbase = Rc::new(LangBaseParser::parse(&langbase_data).map_err(|e| e.to_string())?);

        Ok((decoded.pixels, decoded.palette, sprites, langbase))
    }

    fn load_hills(assets: &AssetStore) -> Result<HillCatalog, String> {
        let data = assets.read(HILLBASE_SKI).map_err(|e| e.to_string())?;
        HillBaseParser::parse(&data).map_err(|e| e.to_string())
    }

    fn load_records(assets: &AssetStore) -> Result<RecordStore, String> {
        let data = assets.read(HISCORE_SKI).map_err(|e| e.to_string())?;
        RecordsParser::parse(&data).map_err(|e| e.to_string())
    }

    fn create_router(
        resources: ResourcesRef,
        background: Vec<u8>,
        store: StoreRef,
    ) -> Router<RouteTarget> {
        let layout = MainLayout::new(
            Rc::clone(&resources.langbase),
            VERSION.to_string(),
            background,
            store.clone(),
        );
        Router::new(
            RouteTarget::Welcome,
            Box::new(WelcomeScreenView::new(
                resources.langbase.languages.clone(),
                &resources.langbase,
            )),
            vec![
                (RouteTarget::MainMenu, {
                    let l = layout.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), &s)))
                }),
                (RouteTarget::JumpMenu, {
                    let l = layout.clone();
                    Box::new(move || Box::new(JumpMenuView::new(l.clone())))
                }),
                (RouteTarget::Practice, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(PracticeView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::Jump, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(JumpView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::CompetitionJump, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(CompetitionJumpView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::Replays, {
                    let r = resources.clone();
                    let s = store.clone();
                    let l = layout.clone();
                    Box::new(move || {
                        Box::new(ReplayBrowserView::new(r.clone(), s.clone(), l.clone()))
                    })
                }),
                (RouteTarget::ReplayPlayback, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(ReplayView::new(r.clone(), &s)))
                }),
                (RouteTarget::ProfilesList, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(ProfilesView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::HallOfFame, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(HallOfFameView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::HillRecords, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(HillRecordsView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::OptionsMenu, {
                    let l = layout.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), &s)))
                }),
                (RouteTarget::Quit, {
                    let l = layout;
                    let s = store;
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), &s)))
                }),
                (RouteTarget::Welcome, {
                    let r = resources;
                    Box::new(move || {
                        Box::new(WelcomeScreenView::new(
                            r.langbase.languages.clone(),
                            &r.langbase,
                        ))
                    })
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
        let palette = {
            let mut p = self.base_palette.clone();
            self.router.apply_palette(&mut p);
            p
        };
        self.renderer.set_palette(palette.clone());

        self.framebuffer.fill(0);
        let mut ctx = engine::ui::PaintCtx::new(&mut self.framebuffer, &palette, WIDTH, HEIGHT);
        let elements = self.router.current_view().elements();
        for el in &elements {
            el.render(&mut ctx, &self.font, &self.sprites);
        }
        self.router
            .current_view()
            .render_snow(&mut self.framebuffer);
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
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = game.run() {
        eprintln!("{e}");
    }
}
