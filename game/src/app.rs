mod assets;
mod rendering;
mod router;
mod state;

use crate::app::router::{create_router, AppRouter};
use crate::files::FileStore;
use crate::route::RouteTarget;
use engine::input::Input;
use engine::oxide::Font;
use engine::sprite::BakedSpriteTextures;
use engine::video::{Renderer, TextureId};
use std::rc::Rc;

use self::assets::LoadedAssets;
use self::rendering::{FrameAssets, FrameRenderer};
use self::state::AppState;

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: AppRouter,
    baked_sprites: BakedSpriteTextures,
    main_background: TextureId,
    pattern_texture: TextureId,
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
            baked_sprites,
            pattern_texture,
        } = assets::load(&files, &mut renderer)?;
        let state = AppState::load(Rc::clone(&files), font.clone(), content_store)?;
        let start_route = {
            let game_state = state.state.borrow();
            let language_count = state.resources.langbase.languages.len() as i32;
            if game_state.config.languagenumber >= 0
                && game_state.config.languagenumber < language_count
            {
                RouteTarget::MainMenu
            } else {
                RouteTarget::Welcome
            }
        };
        let router = create_router(
            state.resources,
            state.state,
            start_route,
            state.save_manager,
        );

        Ok(Self {
            _sdl: sdl,
            renderer,
            input,
            font,
            router,
            baked_sprites,
            main_background,
            pattern_texture,
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
        let font = &self.font;
        self.frame_renderer.render(
            &mut self.renderer,
            FrameAssets {
                font,
                baked_sprites: &self.baked_sprites,
                main_background: self.main_background,
                pattern_texture: self.pattern_texture,
            },
            &mut self.router,
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
