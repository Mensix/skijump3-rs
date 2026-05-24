pub mod competition;
pub mod components;
pub mod content;
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
use crate::save::files::FileStore;
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
use parsers::pcx::PcxParser;
use route::RouteTarget;
use std::rc::Rc;
use views::{
    CustomCupSetupView, HallOfFameView, HillRecordsView, JumpMenuView, MainMenuView, ProfilesView,
    ReplayBrowserView, ReplayView, SetupView, TrainingJumpView, TrainingSetupView,
    WelcomeScreenView, WorldCupJumpView,
};

const MAIN_PCX: &str = "MAIN.PCX";
const ANIM_SKI: &str = "ANIM.SKI";
const CONTENT_MANIFEST: &str = "content.toml";
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

        let save_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let asset_dir = std::path::PathBuf::from("game/assets");
        let files = Rc::new(FileStore::new(asset_dir, save_dir));
        let assets = AssetStore::new(Rc::clone(&files));

        let (pixels, pcx_palette, sprites, content_store) = Self::load_assets(&assets)?;
        let langbase = Rc::new(content_store.langbase);

        let font = Font::from_sprites(&sprites);
        let framebuffer = vec![0u8; (WIDTH * HEIGHT) as usize];

        let mut base_palette = pcx_palette;
        apply_standard_ui_palette(&mut base_palette);
        renderer.set_palette(base_palette.clone());

        let save_manager: SaveRef =
            Rc::new(SaveManager::new(Rc::clone(&files), Rc::clone(&langbase)));

        let start_route = if save_manager.config.borrow().languagenumber == 255 {
            RouteTarget::Welcome
        } else {
            RouteTarget::MainMenu
        };

        let records = assets.load_records(HISCORE_SKI)?;
        let resources: ResourcesRef = Rc::new(Resources::new(
            font.clone(),
            Rc::clone(&langbase),
            content_store.namesets,
            content_store.hills,
            assets,
            Rc::clone(&files),
            save_manager.clone(),
        ));

        let profiles = save_manager.load_players();
        let store: StoreRef = Rc::new(Store::with_profiles(records, profiles));
        store
            .jump_runtime
            .set_wind_place(save_manager.config.borrow().windplace as u8);
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
    ) -> Result<
        (
            Vec<u8>,
            Palette,
            Vec<SpriteData>,
            crate::content::ContentStore,
        ),
        String,
    > {
        let decoded = assets.parse::<PcxParser>(MAIN_PCX)?;
        let sprites = assets.parse::<AnimParser>(ANIM_SKI)?;
        let content = assets.load_content(CONTENT_MANIFEST)?;

        Ok((decoded.pixels, decoded.palette, sprites, content))
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

        // Factory helpers to reduce clone/closure repetition.
        fn rs<F>(
            r: &ResourcesRef,
            s: &StoreRef,
            ctor: F,
        ) -> Box<dyn Fn() -> Box<dyn View<RouteTarget>>>
        where
            F: Fn(ResourcesRef, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
        {
            let r = r.clone();
            let s = s.clone();
            Box::new(move || ctor(r.clone(), s.clone()))
        }

        fn ls<F>(
            l: &MainLayout,
            s: &StoreRef,
            ctor: F,
        ) -> Box<dyn Fn() -> Box<dyn View<RouteTarget>>>
        where
            F: Fn(MainLayout, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
        {
            let l = l.clone();
            let s = s.clone();
            Box::new(move || ctor(l.clone(), s.clone()))
        }

        Router::new(
            start_route,
            initial_view,
            vec![
                (
                    RouteTarget::MainMenu,
                    ls(&layout, &store, |l, s| Box::new(MainMenuView::new(l, s))),
                ),
                (RouteTarget::JumpMenu, {
                    let l = layout.clone();
                    let r = resources.clone();
                    let s = store.clone();
                    Box::new(move || Box::new(JumpMenuView::new(l.clone(), s.clone(), r.clone())))
                }),
                (
                    RouteTarget::Practice,
                    rs(&resources, &store, |r, s| {
                        Box::new(TrainingSetupView::new(r, s))
                    }),
                ),
                (
                    RouteTarget::Jump,
                    rs(&resources, &store, |r, s| {
                        Box::new(TrainingJumpView::new(r, s))
                    }),
                ),
                (
                    RouteTarget::CompetitionJump,
                    rs(&resources, &store, |r, s| {
                        Box::new(WorldCupJumpView::new(r, s))
                    }),
                ),
                (
                    RouteTarget::CustomCupSetup,
                    rs(&resources, &store, |r, s| {
                        Box::new(CustomCupSetupView::new(s, r))
                    }),
                ),
                (RouteTarget::Replays, {
                    let r = resources.clone();
                    let s = store.clone();
                    let l = layout.clone();
                    Box::new(move || {
                        Box::new(ReplayBrowserView::new(r.clone(), s.clone(), l.clone()))
                    })
                }),
                (
                    RouteTarget::ReplayPlayback,
                    rs(&resources, &store, |r, s| Box::new(ReplayView::new(r, s))),
                ),
                (RouteTarget::ProfilesList, {
                    let r = resources.clone();
                    let s = store.clone();
                    let sm = save_manager.clone();
                    Box::new(move || Box::new(ProfilesView::new(r.clone(), s.clone(), sm.clone())))
                }),
                (
                    RouteTarget::HallOfFame,
                    rs(&resources, &store, |r, s| {
                        Box::new(HallOfFameView::new(r, s))
                    }),
                ),
                (
                    RouteTarget::HillRecords,
                    rs(&resources, &store, |r, s| {
                        Box::new(HillRecordsView::new(r, s))
                    }),
                ),
                (
                    RouteTarget::OptionsMenu,
                    rs(&resources, &store, |r, s| Box::new(SetupView::new(r, s))),
                ),
                (
                    RouteTarget::Quit,
                    ls(&layout, &store, |l, s| Box::new(MainMenuView::new(l, s))),
                ),
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
        self.router.current_view_mut().update();

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
