mod router;

use crate::app::router::create_router;
use crate::content::atlas;
use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::gfx::palette::apply_standard_ui_palette;
use crate::gfx::pcx::PcxParser;
use crate::gfx::png::load_png;
use crate::route::RouteTarget;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::atlas::Atlas;
use engine::element_renderer::ElementRenderContext;
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{BackgroundMode, Font, Router};
use engine::video::{Renderer, TextureId};
use std::rc::Rc;

const MAIN_PNG: &str = "MAIN.png";
const MAIN_PCX: &str = "MAIN.PCX";
const CONTENT_MANIFEST: &str = "content.toml";
const HISCORES_TOML: &str = "hiscores.toml";

pub struct Game {
    #[allow(dead_code)]
    sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
    sprites: Vec<SpriteData>,
    sprite_atlas: Option<Atlas>,
    main_background: TextureId,
    element_render_context: ElementRenderContext,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let (sdl, mut renderer, input) = Self::init_sdl()?;

        let save_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let asset_dir = std::path::PathBuf::from("game/assets");
        let files = Rc::new(FileStore::new(asset_dir, save_dir));

        let (pcx_palette, sprites, content_store) = Self::load_assets(&files)?;
        let main_background = Self::load_background_texture(&files, &mut renderer)?;
        let langbase = Rc::new(content_store.langbase);

        let font = Font::from_sprites(&sprites);

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

        let records_data = files.read(HISCORES_TOML).map_err(|e| e.to_string())?;
        let records = RecordStore::from_toml_bytes(&records_data)?;
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
        store
            .jump_runtime
            .set_wind_place(save_manager.config.borrow().windplace as u8);
        let router = create_router(resources, store, start_route, save_manager);

        let sprite_atlas =
            match atlas::load_sprite_atlas(&files, &mut renderer, "sprites/original_atlas.toml") {
                Ok(a) => Some(a),
                Err(e) => {
                    eprintln!("Warning: failed to load sprite atlas: {e}");
                    None
                }
            };

        Ok(Self {
            sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            sprite_atlas,
            main_background,
            element_render_context: ElementRenderContext::new(),
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
        let img = load_png(&png_data)?;
        renderer.create_rgba_texture(&img.pixels, img.width, img.height)
    }

    #[allow(clippy::type_complexity)]
    fn load_assets(files: &FileStore) -> Result<(Palette, Vec<SpriteData>, ContentStore), String> {
        let pcx_data = files.read(MAIN_PCX).map_err(|e| e.to_string())?;
        let decoded = PcxParser::parse(&pcx_data)?;
        let content = ContentStore::load(files, CONTENT_MANIFEST)?;
        let sprites = content.sprites.clone();

        Ok((decoded.palette, sprites, content))
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

        let elements = self.router.current_view().elements();
        let background = match self.router.current_view().gpu_background() {
            BackgroundMode::MainPng => Some(self.main_background),
            BackgroundMode::NoneBlack => None,
        };

        self.element_render_context.render_frame(
            &mut self.renderer,
            &self.font,
            &self.sprites,
            self.sprite_atlas.as_ref(),
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
