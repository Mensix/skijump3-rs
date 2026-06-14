mod router;

use crate::app::router::create_router;
use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::gfx::palette::Rgb6Palette;
use crate::gfx::png::load_png;
use crate::gfx::theme::FONT_HELP;
use crate::route::RouteTarget;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::element_renderer::ElementRenderContext;
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{BackgroundMode, Element, Font, Router};
use engine::video::{Renderer, TextureId};
use std::rc::Rc;
use std::time::Instant;

const MAIN_PNG: &str = "MAIN.png";
const CONTENT_MANIFEST: &str = "content.toml";
const HISCORES_TOML: &str = "hiscores.toml";

pub struct Game {
    _sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
    palette: Palette,
    sprites: Vec<SpriteData>,
    main_background: TextureId,
    element_render_context: ElementRenderContext,
    fps_frame_count: u64,
    fps_elapsed: f64,
    fps_display: f64,
    fps_last: Instant,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let (sdl, mut renderer, input) = Self::init_sdl()?;

        let save_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let asset_dir = std::path::PathBuf::from("game/assets");
        let files = Rc::new(FileStore::new(asset_dir, save_dir));

        let (palette, sprites, content_store) = Self::load_assets(&files)?;
        let main_background = Self::load_background_texture(&files, &mut renderer)?;
        let langbase = Rc::new(content_store.langbase);

        let font = Font::from_sprites(&sprites);

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
            palette,
            sprites,
            main_background,
            element_render_context: ElementRenderContext::new(),
            fps_frame_count: 0,
            fps_elapsed: 0.0,
            fps_display: 0.0,
            fps_last: Instant::now(),
        })
    }

    fn init_sdl() -> Result<(sdl2::Sdl, Renderer, Input), String> {
        let sdl = sdl2::init()?;
        let renderer = Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;
        Ok((sdl, renderer, input))
    }

    fn load_background_texture(
        files: &FileStore,
        renderer: &mut Renderer,
    ) -> Result<TextureId, String> {
        let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
        let img = load_png(&png_data).map_err(|e| e.to_string())?;
        renderer.create_rgba_texture(&img.pixels, img.width, img.height)
    }

    #[allow(clippy::type_complexity)]
    fn load_assets(files: &FileStore) -> Result<(Palette, Vec<SpriteData>, ContentStore), String> {
        let palette_toml = files.read("palette.toml").map_err(|e| e.to_string())?;
        let palette = Rgb6Palette::from_toml_bytes("palette.toml", &palette_toml)
            .map_err(|e| e.to_string())?;
        let content = ContentStore::load(files, CONTENT_MANIFEST).map_err(|e| e.to_string())?;
        let sprites = content.sprites.clone();

        Ok((palette.into_palette(), sprites, content))
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

        let mut elements = self.router.current_view().elements();

        self.fps_frame_count += 1;
        self.fps_elapsed += self.fps_last.elapsed().as_secs_f64();
        self.fps_last = Instant::now();
        if self.fps_elapsed >= 0.5 {
            self.fps_display = self.fps_frame_count as f64 / self.fps_elapsed;
            self.fps_frame_count = 0;
            self.fps_elapsed = 0.0;
        }

        if cfg!(debug_assertions) {
            elements.push(Element::right_text(
                format!("{:.0} fps", self.fps_display),
                319,
                192,
                FONT_HELP,
            ));
        }

        let background = match self.router.current_view().gpu_background() {
            BackgroundMode::MainPng => Some(self.main_background),
            BackgroundMode::NoneBlack => None,
        };

        self.element_render_context.render_frame(
            &mut self.renderer,
            &self.font,
            &self.palette,
            &self.sprites,
            &elements,
            background,
        )?;

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
