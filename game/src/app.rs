mod assets;
mod rendering;
mod router;
mod state;

use crate::app::router::create_router;
use crate::files::FileStore;
use crate::route::RouteTarget;
use engine::platform::Runtime;
use engine::sprite::SpriteData;
use engine::ui::{Font, Router};
use engine::video::TextureId;
use std::rc::Rc;

use self::assets::LoadedAssets;
use self::rendering::FrameRenderer;
use self::state::GameState;

pub struct Game {
    runtime: Runtime,
    font: Font,
    router: Router<RouteTarget>,
    sprites: Vec<SpriteData>,
    main_background: TextureId,
    frame_renderer: FrameRenderer,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let mut runtime = Runtime::new()?;

        let save_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let asset_dir = std::path::PathBuf::from("game/assets");
        let files = Rc::new(FileStore::new(asset_dir, save_dir));

        let LoadedAssets {
            content_store,
            font,
            main_background,
            sprites,
        } = assets::load(&files, runtime.renderer_mut())?;
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
            runtime,
            font,
            router,
            sprites,
            main_background,
            frame_renderer: FrameRenderer::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.runtime.input().running() {
            if self.router.current_route() == Some(&RouteTarget::Quit) {
                break;
            }
            self.handle_input();
            self.render_frame()?;
        }
        Ok(())
    }

    fn handle_input(&mut self) {
        for event in self.runtime.input_mut().drain_events() {
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
        self.frame_renderer.render(
            self.runtime.renderer_mut(),
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
