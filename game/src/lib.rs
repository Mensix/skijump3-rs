pub mod competition;
pub mod components;
pub mod content;
pub mod controllers;
pub mod data;
pub mod files;
pub mod gfx;
pub mod jump;
pub mod rng;
pub mod route;
pub mod save;
pub mod store;
pub mod text;
pub mod views;

use crate::components::layout::MainLayout;
use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::gfx::palette::apply_standard_ui_palette;
use crate::gfx::pcx::PcxParser;
use crate::gfx::png::load_png;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{
    BackgroundMode, Element, Font, PaintCtx, Router, View,
    render_image_bitmap, render_image_region_bitmap,
};
use engine::video::{Renderer, TextureId};
use route::RouteTarget;
use std::rc::Rc;
use views::{
    CustomCupSetupView, HallOfFameView, HillRecordsView, JumpMenuView, MainMenuView, ProfilesView,
    ReplayBrowserView, ReplayView, SetupView, TrainingJumpView, TrainingSetupView,
    WelcomeScreenView, WorldCupJumpView,
};

const MAIN_PNG: &str = "MAIN.png";
const MAIN_PCX: &str = "MAIN.PCX";
const CONTENT_MANIFEST: &str = "content.toml";
const HISCORES_TOML: &str = "hiscores.toml";
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
        store
            .jump_runtime
            .set_wind_place(save_manager.config.borrow().windplace as u8);
        let router = Self::create_router(resources, store, start_route, save_manager);

        Ok(Self {
            sdl,
            renderer,
            input,
            font,
            router,
            sprites,
            framebuffer,
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

    fn load_background_texture(files: &FileStore, renderer: &mut Renderer) -> Result<TextureId, String> {
        let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
        let img = load_png(&png_data)?;
        renderer.create_rgba_texture(&img.pixels, img.width, img.height)
    }

    #[allow(clippy::type_complexity)]
    fn load_assets(
        files: &FileStore,
    ) -> Result<
        (
            Palette,
            Vec<SpriteData>,
            crate::content::ContentStore,
        ),
        String,
    > {
        let pcx_data = files.read(MAIN_PCX).map_err(|e| e.to_string())?;
        let decoded = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;
        let content = ContentStore::load(files, CONTENT_MANIFEST)?;
        let sprites = content.sprites.clone();

        Ok((decoded.palette, sprites, content))
    }

    fn create_router(
        resources: ResourcesRef,
        store: StoreRef,
        start_route: RouteTarget,
        save_manager: SaveRef,
    ) -> Router<RouteTarget> {
        let layout = MainLayout::new(
            Rc::clone(&resources.langbase),
            VERSION.to_string(),
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

        let elements = self.router.current_view().elements();
        let background = match self.router.current_view().gpu_background() {
            BackgroundMode::MainPng => Some(self.main_background),
            BackgroundMode::NoneBlack => None,
        };
        self.render_gpu_frame(&elements, background)?;
        self.renderer.wait_frame();
        Ok(())
    }

    /// Render elements via GPU with an optional background texture.
    /// Pass `Some(texture_id)` for MAIN.png background, or `None` for black.
    /// FillArea-dependent elements fall back to the legacy indexed framebuffer.
    fn render_gpu_frame(
        &mut self,
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        self.renderer.begin_frame();
        if let Some(bg) = background {
            self.renderer.draw_texture(bg, None, None)?;
        }

        self.framebuffer.fill(0);
        let mut dirty = false;
        let mut remaining_fill_areas = count_fill_areas(elements);

        for el in elements {
            self.render_gpu_element(el, &mut dirty, &mut remaining_fill_areas)?;
        }

        if dirty {
            self.renderer.draw_legacy_framebuffer_overlay(&self.framebuffer)?;
        }

        self.renderer.end_frame();
        Ok(())
    }

    /// Recursively process a single element for the GPU rendering path.
    /// `remaining_fill_areas` tracks how many FillArea elements remain
    /// later in the element tree. When > 0, certain elements must render
    /// into the legacy framebuffer so FillArea can read them.
    fn render_gpu_element(
        &mut self,
        element: &Element,
        dirty: &mut bool,
        remaining_fill_areas: &mut usize,
    ) -> Result<(), String> {
        match element {
            Element::Fillbox { .. } | Element::Box { .. } if *remaining_fill_areas > 0 => {
                self.render_into_framebuffer(element, dirty);
            }
            Element::Fillbox { x, y, w, h, color } => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
                self.renderer
                    .draw_indexed_fill_rect(*x, *y, *w, *h, *color)?;
            }
            Element::Box { x, y, w, h, color } => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
                self.renderer
                    .draw_indexed_box(*x, *y, *w, *h, *color)?;
            }
            Element::FillArea { .. } => {
                self.render_into_framebuffer(element, dirty);
                *remaining_fill_areas = remaining_fill_areas.saturating_sub(1);
            }
            Element::Container(children) => {
                for child in children {
                    self.render_gpu_element(child, dirty, remaining_fill_areas)?;
                }
            }
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } if *remaining_fill_areas == 0 => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
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
            Element::Sprite(idx, x, y) if *remaining_fill_areas == 0 => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
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
            Element::SpriteRemapped(idx, x, y, remap) if *remaining_fill_areas == 0 => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
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
            Element::Image(pixels, w, h) if *remaining_fill_areas == 0 => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
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
            Element::ImageRegion(region) if *remaining_fill_areas == 0 => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
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
            Element::Text { .. }
            | Element::Sprite(..)
            | Element::SpriteRemapped(..)
            | Element::Image(..)
            | Element::ImageRegion(..) => {
                self.render_into_framebuffer(element, dirty);
            }
        }
        Ok(())
    }

    /// Flush the legacy overlay if dirty, then clear the framebuffer.
    fn flush_gpu_overlay_if_dirty(&mut self, dirty: &mut bool) -> Result<(), String> {
        if *dirty {
            self.renderer
                .draw_legacy_framebuffer_overlay(&self.framebuffer)?;
            self.framebuffer.fill(0);
            *dirty = false;
        }
        Ok(())
    }

    /// Render an element into the legacy indexed framebuffer.
    fn render_into_framebuffer(&mut self, element: &Element, dirty: &mut bool) {
        if !*dirty {
            self.framebuffer.fill(0);
        }
        let mut ctx = PaintCtx::new(&mut self.framebuffer, WIDTH, HEIGHT);
        element.render(&mut ctx, &self.font, &self.sprites);
        *dirty = true;
    }

}

// ---------------------------------------------------------------------------
// Helper functions for GPU-render element processing
// ---------------------------------------------------------------------------

/// Count how many `FillArea` elements exist in an element tree.
/// Used to decide whether Fillbox/Box can safely use the GPU path.
fn count_fill_areas(elements: &[Element]) -> usize {
    elements.iter().map(|el| count_fill_areas_in_element(el)).sum()
}

fn count_fill_areas_in_element(element: &Element) -> usize {
    match element {
        Element::FillArea { .. } => 1,
        Element::Container(children) => count_fill_areas(children),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_fill_areas_empty() {
        assert_eq!(count_fill_areas(&[]), 0);
    }

    #[test]
    fn count_fill_areas_no_fillareas() {
        let els = vec![Element::fillbox(0, 0, 10, 10, 1), Element::text("hi", 0, 0, 2, false)];
        assert_eq!(count_fill_areas(&els), 0);
    }

    #[test]
    fn count_fill_areas_single() {
        let els = vec![Element::fill_area(64)];
        assert_eq!(count_fill_areas(&els), 1);
    }

    #[test]
    fn count_fill_areas_nested_in_containers() {
        let els = vec![
            Element::container(vec![
                Element::fill_area(63),
                Element::container(vec![
                    Element::fill_area(63),
                    Element::fillbox(0, 0, 10, 10, 1),
                ]),
            ]),
            Element::fill_area(64),
        ];
        assert_eq!(count_fill_areas(&els), 3);
    }

    #[test]
    fn count_fill_areas_mixed() {
        let els = vec![
            Element::fillbox(0, 0, 10, 10, 1),
            Element::text("test", 0, 0, 2, false),
            Element::fill_area(63),
            Element::sprite(0, 0, 0),
        ];
        assert_eq!(count_fill_areas(&els), 1);
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
