mod assets;
mod rendering;
mod router;

use crate::app::router::create_router;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::route::RouteTarget;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::input::Input;
use engine::sprite::SpriteData;
use engine::ui::{Font, Router};
use engine::video::{Renderer, TextureId};
use std::rc::Rc;

use self::assets::LoadedAssets;
use self::rendering::FrameRenderer;

const HISCORES_TOML: &str = "hiscores.toml";

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
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
        let langbase = Rc::new(content_store.langbase);

        let save_manager: SaveRef =
            Rc::new(SaveManager::new(Rc::clone(&files), Rc::clone(&langbase)));

        let start_route = if save_manager.config.borrow().languagenumber == 255 {
            RouteTarget::Welcome
        } else {
            RouteTarget::MainMenu
        };

        let records_data = files.read(HISCORES_TOML).map_err(|e| e.to_string())?;
        let records = RecordStore::from_toml_bytes(&records_data).map_err(|e| e.to_string())?;
        let resources: ResourcesRef = Rc::new(Resources::new(
            font.clone(),
            Rc::clone(&langbase),
            content_store.namesets,
            content_store.hills,
            Rc::clone(&files),
            save_manager.clone(),
        ));

        let profiles = save_manager.load_players();
        let store: StoreRef = Rc::new(Store::with_profiles(records, profiles));
        store.set_wind_place(save_manager.config.borrow().windplace as u8);
        let router = create_router(resources, store, start_route, save_manager);

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
