pub mod competition;
pub mod components;
pub mod controllers;
pub mod data;
pub mod gfx;
pub mod jump;
pub mod loaders;
pub mod parsers;
pub mod rng;
pub mod route;
pub mod save;
pub mod store;
pub mod text;
pub mod views;

use crate::components::layout::MainLayout;
use crate::gfx::palette::apply_standard_ui_palette;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{Font, PaintCtx, Router, View};
use engine::video::Renderer;
use loaders::assets::AssetStore;
use parsers::anim::AnimParser;
use parsers::langbase::{LangBase, LangBaseParser};
use parsers::pcx::PcxParser;
use route::RouteTarget;
use std::rc::Rc;
use views::{
    HallOfFameView, HillRecordsView, JumpMenuView, MainMenuView, ProfilesView, ReplayBrowserView,
    ReplayView, TrainingJumpView, TrainingSetupView, WelcomeScreenView, WorldCupJumpView,
};

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";
const LANGBASE_SKI: &str = "LANGBASE.SKI";
const HILLBASE_SKI: &str = "HILLBASE.SKI";
const HISCORE_SKI: &str = "HISCORE.SKI";
const VERSION: &str = "3.12";

pub struct Game {
    #[allow(dead_code)]
    sdl: sdl2::Sdl,
    renderer: Renderer,
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

        let font = Font::from_sprites(&sprites);
        let framebuffer = vec![0u8; (WIDTH * HEIGHT) as usize];

        let mut base_palette = pcx_palette;
        apply_standard_ui_palette(&mut base_palette);
        renderer.set_palette(base_palette.clone());

        let save_manager: SaveRef = Rc::new(SaveManager::new(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            Rc::clone(&langbase),
        ));

        let start_route = if save_manager.config.borrow().languagenumber == 255 {
            RouteTarget::Welcome
        } else {
            RouteTarget::MainMenu
        };

        let player_names = assets.load_all_names();
        let hills = assets.load_hills(HILLBASE_SKI)?;
        let records = assets.load_records(HISCORE_SKI)?;
        let resources: ResourcesRef = Rc::new(Resources::new(
            font.clone(),
            Rc::clone(&langbase),
            player_names,
            hills,
            assets,
        ));
        let store: StoreRef = Rc::new(Store::new(records));
        let router = Self::create_router(resources, pixels, store, start_route, save_manager);

        Ok(Self {
            sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            framebuffer,
            base_palette,
        })
    }

    fn init_sdl() -> Result<(sdl2::Sdl, Renderer, Input), String> {
        let sdl = sdl2::init()?;
        let renderer = Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;
        Ok((sdl, renderer, input))
    }

    #[allow(clippy::type_complexity)]
    fn load_assets(
        assets: &AssetStore,
    ) -> Result<(Vec<u8>, Palette, Vec<SpriteData>, Rc<LangBase>), String> {
        let decoded = assets.parse::<PcxParser>(MAIN_PCX)?;
        let sprites = assets.parse::<AnimParser>(ANIM_SKI)?;
        let langbase = Rc::new(assets.parse::<LangBaseParser>(LANGBASE_SKI)?);

        Ok((decoded.pixels, decoded.palette, sprites, langbase))
    }

    fn create_router(
        resources: ResourcesRef,
        background: Vec<u8>,
        store: StoreRef,
        start_route: RouteTarget,
        save_manager: SaveRef,
    ) -> Router<RouteTarget> {
        let layout = MainLayout::new(
            Rc::clone(&resources.langbase),
            VERSION.to_string(),
            background,
            store.clone(),
        );
        let initial_view: Box<dyn View<RouteTarget>> = match &start_route {
            RouteTarget::Welcome => Box::new(WelcomeScreenView::new(
                resources.langbase.languages.clone(),
                Rc::clone(&resources.langbase),
                save_manager.clone(),
            )),
            _ => Box::new(MainMenuView::new(layout.clone(), store.clone())),
        };
        Router::new(
            start_route,
            initial_view,
            vec![
                (RouteTarget::MainMenu, {
                    let l = layout.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), s.clone())))
                }),
                (RouteTarget::JumpMenu, {
                    let l = layout.clone();
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(JumpMenuView::new(l.clone(), s.clone(), r.clone())))
                }),
                (RouteTarget::Practice, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(TrainingSetupView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::Jump, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(TrainingJumpView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::CompetitionJump, {
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(WorldCupJumpView::new(r.clone(), s.clone())))
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
                    Box::new(move || Box::new(ReplayView::new(r.clone(), s.clone())))
                }),
                (RouteTarget::ProfilesList, {
                    let r = resources.clone();
                    let s = store.clone();
                    let sm = save_manager.clone();
                    Box::new(move || Box::new(ProfilesView::new(r.clone(), s.clone(), sm.clone())))
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
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), s.clone())))
                }),
                (RouteTarget::Quit, {
                    let l = layout;
                    let s = store;
                    Box::new(move || Box::new(MainMenuView::new(l.clone(), s.clone())))
                }),
                (RouteTarget::Welcome, {
                    let r = resources;
                    let sm = save_manager;
                    Box::new(move || {
                        Box::new(WelcomeScreenView::new(
                            r.langbase.languages.clone(),
                            Rc::clone(&r.langbase),
                            sm.clone(),
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
            if let Some(target) = self.router.current_view_mut().handle_event(event) {
                if target == RouteTarget::Back {
                    self.router.back();
                } else {
                    self.router.navigate(target);
                }
            }
        }
    }

    fn render_frame(&mut self) -> Result<(), String> {
        let palette = {
            let mut p = self.base_palette.clone();
            self.router.apply_palette(&mut p);
            p
        };
        self.renderer.set_palette(palette);

        self.framebuffer.fill(0);
        let mut ctx = PaintCtx::new(&mut self.framebuffer, WIDTH, HEIGHT);
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
