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
use engine::consts::{
    FILL_BRIGHTEN, FILL_RANGE_MAX, HEIGHT, PATTERN_SPRITE, SHADOW_PIXEL, TILE_H, TILE_W, WIDTH,
};
use engine::input::Input;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::ui::{
    render_image_bitmap, render_image_region_bitmap, BackgroundMode, Element, Font, IndexedBitmap,
    Router, View,
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
    overlay_buffer: Vec<u8>,
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
        let overlay_buffer = vec![0u8; (WIDTH * HEIGHT) as usize];

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
            overlay_buffer,
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
    /// FillArea-dependent elements fall back to the indexed overlay buffer.
    fn render_gpu_frame(
        &mut self,
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        self.renderer.begin_frame();
        if let Some(bg) = background {
            self.renderer.draw_texture(bg, None, None)?;
        }

        self.overlay_buffer.fill(0);
        let mut dirty = false;
        let mut remaining_fill_areas = count_fill_areas(elements);

        for el in elements {
            self.render_gpu_element(el, &mut dirty, &mut remaining_fill_areas)?;
        }

        if dirty {
            self.renderer.draw_indexed_overlay(&self.overlay_buffer)?;
        }

        self.renderer.end_frame();
        Ok(())
    }

    /// Recursively process a single element for the GPU rendering path.
    /// `remaining_fill_areas` tracks how many FillArea elements remain
    /// later in the element tree. When > 0, certain elements must render
    /// into the overlay buffer so FillArea can read them.
    fn render_gpu_element(
        &mut self,
        element: &Element,
        dirty: &mut bool,
        remaining_fill_areas: &mut usize,
    ) -> Result<(), String> {
        match element {
            Element::Fillbox { .. } | Element::Box { .. } if *remaining_fill_areas > 0 => {
                self.render_into_overlay_buffer(element, dirty);
            }
            Element::Fillbox { x, y, w, h, color } => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
                self.renderer
                    .draw_indexed_fill_rect(*x, *y, *w, *h, *color)?;
            }
            Element::Box { x, y, w, h, color } => {
                self.flush_gpu_overlay_if_dirty(dirty)?;
                self.renderer.draw_indexed_box(*x, *y, *w, *h, *color)?;
            }
            Element::FillArea { .. } => {
                self.render_into_overlay_buffer(element, dirty);
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
                self.render_into_overlay_buffer(element, dirty);
            }
        }
        Ok(())
    }

    /// Flush the indexed overlay if dirty, then clear the buffer.
    fn flush_gpu_overlay_if_dirty(&mut self, dirty: &mut bool) -> Result<(), String> {
        if *dirty {
            self.renderer.draw_indexed_overlay(&self.overlay_buffer)?;
            self.overlay_buffer.fill(0);
            *dirty = false;
        }
        Ok(())
    }

    /// Render an element into the indexed overlay buffer for FillArea processing.
    fn render_into_overlay_buffer(&mut self, element: &Element, dirty: &mut bool) {
        if !*dirty {
            self.overlay_buffer.fill(0);
        }
        render_element_to_overlay_buffer(
            element,
            &mut self.overlay_buffer,
            &self.font,
            &self.sprites,
        );
        *dirty = true;
    }
}

// ---------------------------------------------------------------------------
// Helper functions for GPU-render element processing
// ---------------------------------------------------------------------------

/// Draw a filled rectangle into an indexed overlay buffer.
/// Clips to screen bounds. Negative or zero w/h are handled gracefully.
fn fill_rect_to_overlay_buffer(buffer: &mut [u8], x: i32, y: i32, w: i32, h: i32, color: u8) {
    let left = x.max(0);
    let top = y.max(0);
    let right = (x + w).min(WIDTH as i32);
    let bottom = (y + h).min(HEIGHT as i32);
    let w = (right - left).max(0) as usize;
    let h = (bottom - top).max(0) as usize;
    let x = left as usize;
    let y = top as usize;
    for dy in 0..h {
        let idx = (y + dy) * (WIDTH as usize) + x;
        buffer[idx..idx + w].fill(color);
    }
}

/// Copy an `IndexedBitmap` into an indexed overlay buffer.
/// When `transparent_zero` is true, pixels with value 0 are skipped.
/// When false, all pixels (including 0) are copied as-is.
fn blit_bitmap_to_overlay_buffer(buffer: &mut [u8], bm: &IndexedBitmap, transparent_zero: bool) {
    let bw = WIDTH as usize;
    for yy in 0..bm.height as i32 {
        for xx in 0..bm.width as i32 {
            let pixel = bm.pixels[(yy as usize) * (bm.width as usize) + (xx as usize)];
            if transparent_zero && pixel == 0 {
                continue;
            }
            let dx = (bm.x + xx) as usize;
            let dy = (bm.y + yy) as usize;
            if dx < bw && dy < HEIGHT as usize {
                buffer[dy * bw + dx] = pixel;
            }
        }
    }
}

/// Dispatch an Element directly into an indexed overlay buffer.
fn render_element_to_overlay_buffer(
    element: &Element,
    buffer: &mut [u8],
    font: &Font,
    sprites: &[SpriteData],
) {
    match element {
        Element::Fillbox { x, y, w, h, color } => {
            fill_rect_to_overlay_buffer(buffer, *x, *y, *w, *h, *color);
        }
        Element::Box { x, y, w, h, color } => {
            let left = (*x).max(0);
            let top = (*y).max(0);
            let right = (*x + *w).min(WIDTH as i32);
            let bottom = (*y + *h).min(HEIGHT as i32);
            let w = (right - left).max(0) as usize;
            let h = (bottom - top).max(0) as usize;
            let x = left as usize;
            let y = top as usize;
            if w == 0 || h == 0 {
                return;
            }
            let x = x as usize;
            let y = y as usize;
            let bw = WIDTH as usize;
            let top = y * bw + x;
            buffer[top..top + w].fill(*color);
            let bot = (y + h - 1) * bw + x;
            buffer[bot..bot + w].fill(*color);
            for dy in 1..h.saturating_sub(1) {
                let idx = (y + dy) * bw + x;
                buffer[idx] = *color;
            }
            if w > 1 {
                for dy in 1..h.saturating_sub(1) {
                    let idx = (y + dy) * bw + x + w - 1;
                    buffer[idx] = *color;
                }
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
            let text_w = font.string_width(text) as i32;
            let fx = if *center {
                x - text_w / 2
            } else if *right {
                x - text_w
            } else {
                *x
            };
            if let Some(bm) = font.render_string_bitmap(text, fx, *y, *color) {
                blit_bitmap_to_overlay_buffer(buffer, &bm, true);
            }
        }
        Element::Sprite(idx, x, y) => {
            if let Some(sprite) = sprites.get(*idx as usize) {
                if let Some(bm) = sprite.render_bitmap(*x, *y) {
                    blit_bitmap_to_overlay_buffer(buffer, &bm, true);
                }
            }
        }
        Element::SpriteRemapped(idx, x, y, remap) => {
            if let Some(sprite) = sprites.get(*idx as usize) {
                if let Some(bm) = sprite.render_bitmap_with_remap(*x, *y, remap) {
                    blit_bitmap_to_overlay_buffer(buffer, &bm, true);
                }
            }
        }
        Element::Image(pixels, w, h) => {
            let Some(bm) = render_image_bitmap(pixels, *w, *h) else {
                return;
            };
            blit_bitmap_to_overlay_buffer(buffer, &bm, false);
        }
        Element::ImageRegion(region) => {
            let Some(bm) = render_image_region_bitmap(region) else {
                return;
            };
            blit_bitmap_to_overlay_buffer(buffer, &bm, false);
        }
        Element::FillArea { thing } => {
            let Some(pattern) = sprites.get(PATTERN_SPRITE) else {
                return;
            };
            let bw = WIDTH as usize;
            for py in 0..HEIGHT as usize {
                for px in 0..WIDTH as usize {
                    let cur = buffer[py * bw + px];
                    if cur <= SHADOW_PIXEL || cur > FILL_RANGE_MAX {
                        continue;
                    }
                    let (ax, ay) = if *thing == 64 {
                        ((px + 2) % TILE_W as usize, (py + 7) % TILE_H as usize)
                    } else {
                        (px % TILE_W as usize, py % TILE_H as usize)
                    };
                    let pi = ay * TILE_W as usize + ax;
                    if pi < pattern.data.len() && pattern.data[pi] != 0 {
                        buffer[py * bw + px] = cur + FILL_BRIGHTEN;
                    }
                }
            }
        }
        Element::Container(children) => {
            for child in children {
                render_element_to_overlay_buffer(child, buffer, font, sprites);
            }
        }
    }
}

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

    // -----------------------------------------------------------------------
    // Indexed overlay buffer tests
    // -----------------------------------------------------------------------

    #[test]
    fn fill_rect_writes_color() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, 5, 5, 10, 10, 7);
        for y in 5..15 {
            let idx = y * (WIDTH as usize) + 5;
            assert_eq!(&buf[idx..idx + 10], &[7u8; 10]);
        }
    }

    #[test]
    fn fill_rect_clips_left_edge() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, -5, 0, 10, 1, 7);
        // Visible from x=0..5
        assert_eq!(buf[..5], [7u8; 5]);
        assert_eq!(buf[5], 0);
    }

    #[test]
    fn fill_rect_clips_right_edge() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, WIDTH as i32 - 3, 0, 10, 1, 7);
        // Only 3 pixels visible
        assert_eq!(buf[(WIDTH as usize - 3)..WIDTH as usize], [7u8; 3]);
    }

    #[test]
    fn fill_rect_clips_top_edge() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, 0, -5, 1, 10, 7);
        // Visible from y=0..5
        for y in 0..5 {
            assert_eq!(buf[y * (WIDTH as usize)], 7, "y={y}");
        }
        assert_eq!(buf[5 * (WIDTH as usize)], 0);
    }

    #[test]
    fn fill_rect_fully_offscreen_left_is_noop() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, -100, 0, 10, 10, 7);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn fill_rect_zero_size_is_noop() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        fill_rect_to_overlay_buffer(&mut buf, 0, 0, 0, 10, 7);
        assert!(buf.iter().all(|&b| b == 0));
        fill_rect_to_overlay_buffer(&mut buf, 0, 0, 10, 0, 7);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn blit_bitmap_transparent_zero_true_skips_zero() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        let bm = IndexedBitmap {
            pixels: vec![0u8, 1, 2, 0],
            x: 10,
            y: 10,
            width: 4,
            height: 1,
        };
        blit_bitmap_to_overlay_buffer(&mut buf, &bm, true);
        // Only pixel indices 1 and 2 are written
        let idx = 10 * (WIDTH as usize) + 10;
        assert_eq!(buf[idx], 0, "index 0 skipped");
        assert_eq!(buf[idx + 1], 1);
        assert_eq!(buf[idx + 2], 2);
        assert_eq!(buf[idx + 3], 0, "index 0 skipped");
    }

    #[test]
    fn blit_bitmap_transparent_zero_false_copies_zero() {
        let mut buf = vec![99u8; (WIDTH * HEIGHT) as usize];
        let bm = IndexedBitmap {
            pixels: vec![0u8, 1],
            x: 5,
            y: 5,
            width: 2,
            height: 1,
        };
        blit_bitmap_to_overlay_buffer(&mut buf, &bm, false);
        let idx = 5 * (WIDTH as usize) + 5;
        assert_eq!(buf[idx], 0, "index 0 copied");
        assert_eq!(buf[idx + 1], 1);
    }

    #[test]
    fn fill_area_brightens_only_eligible_indices() {
        // Set up a minimal pattern sprite at PATTERN_SPRITE (62).
        // Only pixel (0,0) of the 19x13 tile is non-zero.
        let mut sprites = vec![
            SpriteData {
                data: vec![],
                width: 0,
                height: 0,
                center_x: 0,
                center_y: 0
            };
            63
        ];
        sprites[PATTERN_SPRITE] = SpriteData {
            data: {
                let mut d = vec![0u8; (TILE_W * TILE_H) as usize];
                d[0] = 1;
                d
            },
            width: TILE_W as u16,
            height: TILE_H as u16,
            center_x: 0,
            center_y: 0,
        };

        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        // Pixel at tile origin (0,0) — pattern hit — with eligible value
        buf[0] = SHADOW_PIXEL + 1; // 243, in (SHADOW_PIXEL ..= FILL_RANGE_MAX]

        // Pixel at (TILE_W, 0) — also pattern hit (multiple of TILE_W) — with boundary values
        // But we only have 1 column of data, so let's use a miss position
        buf[1] = FILL_RANGE_MAX; // 245, eligible but pattern[1]=0 → miss

        // Pixel above FILL_RANGE_MAX
        buf[2] = FILL_RANGE_MAX + 1; // 246, > FILL_RANGE_MAX → skip

        let el = Element::FillArea { thing: 63 };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &sprites);

        assert_eq!(
            buf[0],
            SHADOW_PIXEL + 1 + FILL_BRIGHTEN,
            "eligible value + pattern hit → brightened"
        );
        assert_eq!(
            buf[1], FILL_RANGE_MAX,
            "eligible + pattern miss → unchanged"
        );
        assert_eq!(buf[2], FILL_RANGE_MAX + 1, "above range → unchanged");

        // Pixel (0,1): eligible (243) + pattern miss (ay=1, pattern[19]=0) → unchanged
        buf[WIDTH as usize] = SHADOW_PIXEL + 1;
        let el2 = Element::FillArea { thing: 63 };
        render_element_to_overlay_buffer(&el2, &mut buf, &Font::new(), &sprites);
        assert_eq!(
            buf[WIDTH as usize],
            SHADOW_PIXEL + 1,
            "eligible + pattern miss (y offset) → unchanged"
        );
    }

    #[test]
    fn fill_area_skips_shadow_pixel_at_hit_position() {
        let mut sprites = vec![
            SpriteData {
                data: vec![],
                width: 0,
                height: 0,
                center_x: 0,
                center_y: 0
            };
            63
        ];
        sprites[PATTERN_SPRITE] = SpriteData {
            data: {
                let mut d = vec![0u8; (TILE_W * TILE_H) as usize];
                d[0] = 1;
                d
            },
            width: TILE_W as u16,
            height: TILE_H as u16,
            center_x: 0,
            center_y: 0,
        };

        let mut buf = vec![SHADOW_PIXEL; (WIDTH * HEIGHT) as usize];
        // Pixel at (0,0) with value SHADOW_PIXEL: pattern hits but pixel is exactly
        // at the SHADOW_PIXEL boundary → skipped (cur <= SHADOW_PIXEL)
        let el = Element::FillArea { thing: 63 };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &sprites);
        assert_eq!(
            buf[0], SHADOW_PIXEL,
            "SHADOW_PIXEL at pattern hit → not brightened"
        );
    }

    #[test]
    fn fill_area_thing_64_uses_shifted_pattern() {
        let mut sprites = vec![
            SpriteData {
                data: vec![],
                width: 0,
                height: 0,
                center_x: 0,
                center_y: 0
            };
            63
        ];
        sprites[PATTERN_SPRITE] = SpriteData {
            data: {
                let mut d = vec![0u8; (TILE_W * TILE_H) as usize];
                d[0] = 1; // pattern hit only at top-left of tile
                d
            },
            width: TILE_W as u16,
            height: TILE_H as u16,
            center_x: 0,
            center_y: 0,
        };

        let mut buf = vec![SHADOW_PIXEL + 1; (WIDTH * HEIGHT) as usize];

        // With thing=64, coordinates are shifted: (px+2)%TILE_W, (py+7)%TILE_H
        // Pixel at (17, 6): ax = (17+2)%19 = 0, ay = (6+7)%13 = 0 → pattern[0] = 1 → brightened
        // Pixel at (0, 0):  ax = (0+2)%19  = 2, ay = (0+7)%13  = 7 → pattern[7*19+2] = 0 → not brightened
        let el = Element::FillArea { thing: 64 };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &sprites);

        let idx_hit = 6 * (WIDTH as usize) + 17;
        assert_eq!(
            buf[idx_hit],
            SHADOW_PIXEL + 1 + FILL_BRIGHTEN,
            "shifted hit brightened"
        );

        let idx_miss = 0 * (WIDTH as usize) + 0;
        assert_eq!(
            buf[idx_miss],
            SHADOW_PIXEL + 1,
            "shifted miss not brightened"
        );
    }

    #[test]
    fn fillbox_element_uses_fill_rect_helper() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        let el = Element::Fillbox {
            x: 7,
            y: 8,
            w: 5,
            h: 3,
            color: 12,
        };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &[]);
        for dy in 0..3 {
            let idx = (8 + dy) * (WIDTH as usize) + 7;
            assert_eq!(&buf[idx..idx + 5], &[12u8; 5], "fillbox row {dy}");
        }
    }

    #[test]
    fn fillbox_clips_extreme_negative() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        // x=-100 with w=1000: clipped to x=0, w=WIDTH
        let el = Element::Fillbox {
            x: -100,
            y: -100,
            w: 1000,
            h: 1000,
            color: 99,
        };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &[]);
        // Full screen should be filled
        assert!(buf.iter().all(|&p| p == 99));
    }

    #[test]
    fn box_clips_extreme_negative() {
        let mut buf = vec![0u8; (WIDTH * HEIGHT) as usize];
        let el = Element::Box {
            x: -100,
            y: -100,
            w: 1000,
            h: 1000,
            color: 99,
        };
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &[]);
        // Top and bottom edges fill the screen rows
        let mut idx_row0 = buf[..WIDTH as usize].iter();
        let mut idx_row_last = buf[(HEIGHT as usize - 1) * (WIDTH as usize)..].iter();
        assert!(idx_row0.all(|&p| p == 99), "top edge");
        assert!(idx_row_last.all(|&p| p == 99), "bottom edge");
    }

    #[test]
    fn image_overlay_copies_zero_as_content() {
        let pixels: Vec<u8> = vec![0u8; 40]; // 10x4 all-zero image
        let buf_size = (WIDTH * HEIGHT) as usize;
        let mut buf = vec![99u8; buf_size];

        let el = Element::Image(Rc::from(pixels), 10, 4);
        render_element_to_overlay_buffer(&el, &mut buf, &Font::new(), &[]);
        // All pixels should be overwritten with 0 (not skipped)
        for y in 0..4 {
            let idx = y * (WIDTH as usize);
            assert_eq!(&buf[idx..idx + 10], &[0u8; 10], "image row {y}");
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
