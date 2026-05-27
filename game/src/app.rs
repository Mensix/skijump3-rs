mod render;
mod router;

use crate::app::render::{count_fill_areas, is_fill_area_dither_color, DitherRect};
use crate::app::router::create_router;
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
use engine::consts::PATTERN_SPRITE;
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{
    render_image_bitmap, render_image_region_bitmap, BackgroundMode, Element, Font, Router,
};
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
    pending_dither_rects: Vec<DitherRect>,
    base_palette: Palette,
    main_background: TextureId,
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

        let sprite_atlas = crate::content::atlas::load_sprite_atlas(
            &files,
            &mut renderer,
            "sprites/original_atlas.toml",
        )
        .ok();

        Ok(Self {
            sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            sprite_atlas,
            pending_dither_rects: Vec::new(),
            base_palette,
            main_background,
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
    fn load_assets(
        files: &FileStore,
    ) -> Result<(Palette, Vec<SpriteData>, crate::content::ContentStore), String> {
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

        let palette = {
            let mut p = self.base_palette.clone();
            self.router.apply_palette(&mut p);
            p
        };
        self.renderer.set_palette(palette);

        let elements = self.router.current_view().elements();
        let background = match self.router.current_view().gpu_background() {
            BackgroundMode::MainPng => Some(self.main_background),
            BackgroundMode::NoneBlack => None,
        };
        self.render_gpu_frame(&elements, background)?;
        self.renderer.wait_frame();
        Ok(())
    }

    fn render_gpu_frame(
        &mut self,
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        self.pending_dither_rects.clear();
        self.renderer.begin_frame();
        if let Some(bg) = background {
            self.renderer.draw_texture(bg, None, None)?;
        }

        let mut remaining_fill_areas = count_fill_areas(elements);

        for el in elements {
            self.render_gpu_element(el, &mut remaining_fill_areas)?;
        }

        self.renderer.end_frame();
        Ok(())
    }

    fn render_gpu_element(
        &mut self,
        element: &Element,
        remaining_fill_areas: &mut usize,
    ) -> Result<(), String> {
        match element {
            Element::Fillbox { x, y, w, h, color } if *remaining_fill_areas > 0 => {
                self.renderer
                    .draw_indexed_fill_rect(*x, *y, *w, *h, *color)?;
                if is_fill_area_dither_color(*color) {
                    self.pending_dither_rects.push(DitherRect {
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                        color: *color,
                        is_box: false,
                    });
                }
            }
            Element::Box { x, y, w, h, color } if *remaining_fill_areas > 0 => {
                self.renderer.draw_indexed_box(*x, *y, *w, *h, *color)?;
                if is_fill_area_dither_color(*color) {
                    self.pending_dither_rects.push(DitherRect {
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                        color: *color,
                        is_box: true,
                    });
                }
            }
            Element::Fillbox { x, y, w, h, color } => {
                self.renderer
                    .draw_indexed_fill_rect(*x, *y, *w, *h, *color)?;
            }
            Element::Box { x, y, w, h, color } => {
                self.renderer.draw_indexed_box(*x, *y, *w, *h, *color)?;
            }
            Element::FillArea { thing } => {
                if let Some(pattern) = self.sprites.get(PATTERN_SPRITE) {
                    for dr in &self.pending_dither_rects {
                        self.renderer.dither_overlay_rect(
                            dr.x,
                            dr.y,
                            dr.w,
                            dr.h,
                            dr.color,
                            dr.is_box,
                            *thing,
                            &pattern.data,
                        )?;
                    }
                }
                if !self.pending_dither_rects.is_empty() {
                    self.renderer.flush_dither_overlay()?;
                    self.pending_dither_rects.clear();
                }
                *remaining_fill_areas = remaining_fill_areas.saturating_sub(1);
            }
            Element::Container(children) => {
                for child in children {
                    self.render_gpu_element(child, remaining_fill_areas)?;
                }
            }
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } => {
                let text_w = self.font.string_width(text) as i32;
                let fx = if *center {
                    x - text_w / 2
                } else if *right {
                    x - text_w
                } else {
                    *x
                };
                if let Some(bitmap) = self.font.render_string_bitmap(text, fx, *y, *color) {
                    self.renderer.draw_indexed_overlay_pixels(
                        &bitmap.pixels,
                        bitmap.width,
                        bitmap.height,
                        bitmap.x,
                        bitmap.y,
                    )?;
                }
            }
            Element::Sprite(idx, x, y) => {
                let drew_from_atlas = self.sprite_atlas.as_ref().is_some_and(|atlas| {
                    atlas.region(*idx as usize).is_some_and(|region| {
                        self.renderer
                            .draw_atlas_region(atlas.texture_id, region, *x, *y)
                            .is_ok()
                    })
                });
                if !drew_from_atlas {
                    if let Some(sprite) = self.sprites.get(*idx as usize) {
                        if let Some(bitmap) = sprite.render_bitmap(*x, *y) {
                            self.renderer.draw_indexed_overlay_pixels(
                                &bitmap.pixels,
                                bitmap.width,
                                bitmap.height,
                                bitmap.x,
                                bitmap.y,
                            )?;
                        }
                    }
                }
            }
            Element::SpriteRemapped(idx, x, y, remap) => {
                if let Some(sprite) = self.sprites.get(*idx as usize) {
                    if let Some(bitmap) = sprite.render_bitmap_with_remap(*x, *y, remap) {
                        self.renderer.draw_indexed_overlay_pixels(
                            &bitmap.pixels,
                            bitmap.width,
                            bitmap.height,
                            bitmap.x,
                            bitmap.y,
                        )?;
                    }
                }
            }
            Element::Image(pixels, w, h) => {
                if let Some(bitmap) = render_image_bitmap(pixels, *w, *h) {
                    self.renderer.draw_indexed_overlay_pixels(
                        &bitmap.pixels,
                        bitmap.width,
                        bitmap.height,
                        bitmap.x,
                        bitmap.y,
                    )?;
                }
            }
            Element::ImageRegion(region) => {
                if let Some(bitmap) = render_image_region_bitmap(region) {
                    self.renderer.draw_indexed_overlay_pixels(
                        &bitmap.pixels,
                        bitmap.width,
                        bitmap.height,
                        bitmap.x,
                        bitmap.y,
                    )?;
                }
            }
        }
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
