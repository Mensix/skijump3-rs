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
use engine::consts::{FILL_RANGE_MAX, PATTERN_SPRITE, SHADOW_PIXEL};
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{
    render_image_bitmap, render_image_region_bitmap, BackgroundMode, Element, Font, Router, View,
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

struct DitherRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u8,
    is_box: bool,
}

pub struct Game {
    #[allow(dead_code)]
    sdl: sdl2::Sdl,
    renderer: Renderer,
    input: Input,
    font: Font,
    router: Router<RouteTarget>,
    sprites: Vec<SpriteData>,
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
    fn render_gpu_frame(
        &mut self,
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
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

    /// Recursively process a single element for the GPU rendering path.
    /// `remaining_fill_areas` tracks how many FillArea elements remain
    /// later in the element tree. When > 0, eligible Fillbox/Box rects
    /// are drawn via GPU and also tracked in `pending_dither_rects`.
    fn render_gpu_element(
        &mut self,
        element: &Element,
        remaining_fill_areas: &mut usize,
    ) -> Result<(), String> {
        match element {
            Element::Fillbox { x, y, w, h, color } if *remaining_fill_areas > 0 => {
                self.renderer
                    .draw_indexed_fill_rect(*x, *y, *w, *h, *color)?;
                if *color > SHADOW_PIXEL && *color <= FILL_RANGE_MAX {
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
                if *color > SHADOW_PIXEL && *color <= FILL_RANGE_MAX {
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
                    let pw = pattern.width as u32;
                    let ph = pattern.height as u32;
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
                            pw,
                            ph,
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

// ---------------------------------------------------------------------------
// Helper functions for GPU-render element processing
// ---------------------------------------------------------------------------

/// Count how many `FillArea` elements exist in an element tree.
/// Used to decide whether Fillbox/Box can safely use the GPU path.
fn count_fill_areas(elements: &[Element]) -> usize {
    elements
        .iter()
        .map(|el| count_fill_areas_in_element(el))
        .sum()
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
        let els = vec![
            Element::fillbox(0, 0, 10, 10, 1),
            Element::text("hi", 0, 0, 2, false),
        ];
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

    #[test]
    fn dither_rect_tracks_eligible_rects_in_pending() {
        // Verify that the DitherRect struct tracks the right fields
        let dr = DitherRect {
            x: 10,
            y: 20,
            w: 30,
            h: 40,
            color: 243,
            is_box: false,
        };
        assert_eq!(dr.x, 10);
        assert_eq!(dr.y, 20);
        assert_eq!(dr.w, 30);
        assert_eq!(dr.h, 40);
        assert_eq!(dr.color, 243);
        assert_eq!(dr.is_box, false);
    }

    #[test]
    fn dither_rect_box_default_is_box() {
        let dr = DitherRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
            color: 244,
            is_box: true,
        };
        assert!(dr.is_box);
        assert_eq!(dr.color, 244);
    }

    #[test]
    fn eligible_color_range_check() {
        // Colors eligible for dither: SHADOW_PIXEL < color <= FILL_RANGE_MAX
        // i.e., 243, 244, 245
        let eligible = [243u8, 244, 245];
        let ineligible = [0u8, 100, SHADOW_PIXEL, FILL_RANGE_MAX + 1];
        for &c in &eligible {
            assert!(
                c > SHADOW_PIXEL && c <= FILL_RANGE_MAX,
                "color {c} should be eligible"
            );
        }
        for &c in &ineligible {
            assert!(
                !(c > SHADOW_PIXEL && c <= FILL_RANGE_MAX),
                "color {c} should be ineligible"
            );
        }
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
