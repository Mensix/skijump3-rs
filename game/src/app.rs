mod assets;
mod rendering;
mod router;
mod state;

use crate::app::router::{create_router, AppRouter};
use crate::files::FileStore;
use crate::route::RouteTarget;
use engine::input::Input;
use engine::oxide::Font;
use engine::sprite::SpriteData;
use engine::video::{Renderer, TextureId};
use std::rc::Rc;

use self::assets::LoadedAssets;
use self::rendering::FrameRenderer;
use self::state::GameState;

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: AppRouter,
    sprites: Vec<SpriteData>,
    main_background: TextureId,
    frame_renderer: FrameRenderer,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let (sdl, mut renderer, input) = Self::init_sdl()?;

        let save_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let asset_dir = std::path::PathBuf::from("game/assets");
        let files = Rc::new(FileStore::new(asset_dir, save_dir));

        let LoadedAssets {
            content_store,
            font,
            main_background,
            sprites,
        } = assets::load(&files, &mut renderer)?;
        let state = GameState::load(Rc::clone(&files), font.clone(), content_store)?;
        let start_route = if state.starts_with_welcome() {
            RouteTarget::Welcome
        } else {
            RouteTarget::MainMenu
        };
        let router = create_router(
            state.resources,
            state.store,
            start_route,
            state.save_manager,
        );

        Ok(Self {
            _sdl: sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            main_background,
            frame_renderer: FrameRenderer::new(),
        })
    }

    fn init_sdl() -> Result<(sdl2::Sdl, Renderer, Input), String> {
        let sdl = sdl2::init()?;
        let renderer = Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;
        Ok((sdl, renderer, input))
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
            self.router.handle_event(event);
        }
    }

    fn render_frame(&mut self) -> Result<(), String> {
        self.router.update();
        self.frame_renderer.render(
            &mut self.renderer,
            &self.font,
            &self.sprites,
            &self.router,
            self.main_background,
        )
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
