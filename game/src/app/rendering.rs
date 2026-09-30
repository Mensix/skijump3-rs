use crate::app::router::AppRouter;
use crate::gfx::materials::to_engine_material;
use crate::gfx::theme::FONT_GRAY;
use crate::ui::{
    MenuRenderSnapshot, ScreenBackground, SpriteMaterialData, StaticImage, TextInputRenderSnapshot,
    UiCanvas,
};
use engine::color::Rgba;
use engine::oxide::{CommandBuffer, DrawRenderAssets, Font, OxideRenderer, PaintCx};
use engine::sprite::BakedSpriteTextures;
use engine::video::{Renderer, TextureId};

pub(super) struct FrameRenderer {
    renderer: OxideRenderer,
    commands: CommandBuffer,
    fps: FpsCounter,
}

impl FrameRenderer {
    pub(super) fn new() -> Self {
        Self {
            renderer: OxideRenderer::new(),
            commands: CommandBuffer::new(),
            fps: FpsCounter::new(),
        }
    }

    pub(super) fn render(
        &mut self,
        renderer: &mut Renderer,
        assets: FrameAssets<'_>,
        router: &mut AppRouter,
    ) {
        {
            let cx = PaintCx::new(&mut self.commands, assets.font);
            let mut canvas = OxideCanvas { cx };
            router.paint(&mut canvas);
            Self::add_debug_overlay(&mut self.fps, &mut canvas);
        }

        let background = match router.screen_background() {
            ScreenBackground::MainPng => Some(assets.main_background),
            ScreenBackground::NoneBlack => None,
        };

        self.renderer.render_commands(
            renderer,
            DrawRenderAssets {
                font: assets.font,
                baked_sprites: assets.baked_sprites,
                pattern_texture: assets.pattern_texture,
            },
            &self.commands,
            background,
        );
        renderer.wait_frame();
    }

    pub(super) fn prepare_frame(&mut self) {
        self.commands.clear();
    }

    fn add_debug_overlay(fps_counter: &mut FpsCounter, cx: &mut dyn UiCanvas) {
        let fps = fps_counter.tick();
        if cfg!(debug_assertions) {
            cx.right_text((319, 192), FONT_GRAY, &format!("{fps:.0} fps"));
        }
    }
}

struct OxideCanvas<'a> {
    cx: PaintCx<'a>,
}

impl UiCanvas for OxideCanvas<'_> {
    fn string_width(&self, text: &str) -> u32 {
        self.cx.string_width(text)
    }
    fn fill(&mut self, rect: (i32, i32, i32, i32), color: Rgba) {
        self.cx.fill(rect, color);
    }
    fn stroke(&mut self, rect: (i32, i32, i32, i32), color: Rgba) {
        self.cx.stroke(rect, color);
    }
    fn pattern_fill(&mut self, rect: (i32, i32, i32, i32), color: Rgba) {
        self.cx.pattern_fill(rect, color);
    }
    fn text(&mut self, position: (i32, i32), color: Rgba, text: &str) {
        self.cx.text(position, color, text);
    }
    fn right_text(&mut self, position: (i32, i32), color: Rgba, text: &str) {
        self.cx.right_text(position, color, text);
    }
    fn center_text(&mut self, position: (i32, i32), color: Rgba, text: &str) {
        self.cx.center_text(position, color, text);
    }
    fn sprite(&mut self, idx: u16, position: (i32, i32)) {
        self.cx.sprite(idx, position);
    }
    fn sprite_with_material(
        &mut self,
        idx: u16,
        position: (i32, i32),
        material: SpriteMaterialData,
    ) {
        self.cx
            .sprite_with_material(idx, position, to_engine_material(material));
    }
    fn static_image_region(
        &mut self,
        image: StaticImage,
        source: engine::oxide::Rect,
        destination: engine::oxide::Rect,
        modulation: Option<Rgba>,
        destination_offset: (i32, i32),
    ) {
        self.cx
            .static_image_region(image, source, destination, modulation, destination_offset);
    }

    fn pixels(&mut self, batches: engine::oxide::PointBatches) {
        self.cx.pixels(batches);
    }

    fn paint_pixel_menu(&mut self, menu: &MenuRenderSnapshot) {
        let mut gap = 0;
        for (i, item) in menu.items.iter().enumerate() {
            gap += item.gap_before;
            if menu.show_labels {
                let y = menu.y + 1 + i as i32 * menu.item_h + gap + item.y_offset;
                self.cx.text(
                    (menu.x, y),
                    menu.font_color,
                    format!("{} - {}", item.number, item.label),
                );
            }
        }
        if menu.show_box && menu.selected < menu.items.len() {
            let item_y = menu_item_y(menu, menu.selected);
            self.cx.stroke(
                (menu.x - 6, item_y - 3, menu.item_w + 1, menu.item_h + 1),
                menu.box_color,
            );
        }
    }

    fn paint_text_input(&mut self, input: &TextInputRenderSnapshot) {
        let cursor_x = input.x + self.cx.string_width(&input.text) as i32;
        self.cx.fill(
            (input.x - 2, input.y - 2, input.max_width + 4, 10),
            input.bg,
        );
        self.cx.text((input.x, input.y), input.fg, &input.text);
        if input.cursor_visible {
            self.cx.fill((cursor_x, input.y + 6, 5, 1), input.cursor);
        }
    }
}

fn menu_item_y(menu: &MenuRenderSnapshot, selected: usize) -> i32 {
    let gap: i32 = menu.items[..=selected]
        .iter()
        .map(|item| item.gap_before)
        .sum();
    menu.y + selected as i32 * menu.item_h + gap + menu.items[selected].y_offset
}

pub(super) struct FrameAssets<'a> {
    pub(super) font: &'a Font,
    pub(super) baked_sprites: &'a BakedSpriteTextures,
    pub(super) main_background: TextureId,
    pub(super) pattern_texture: TextureId,
}

struct FpsCounter {
    frame_count: u64,
    elapsed: f64,
    display: f64,
    last: std::time::Instant,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed: 0.0,
            display: 0.0,
            last: std::time::Instant::now(),
        }
    }

    fn tick(&mut self) -> f64 {
        self.frame_count += 1;
        self.elapsed += self.last.elapsed().as_secs_f64();
        self.last = std::time::Instant::now();

        if self.elapsed >= 0.5 {
            self.display = self.frame_count as f64 / self.elapsed;
            self.frame_count = 0;
            self.elapsed = 0.0;
        }

        self.display
    }
}

#[cfg(test)]
mod tests {
    use super::menu_item_y;
    use crate::ui::{MenuItemRenderSnapshot, MenuRenderSnapshot};
    use engine::color::Rgba;

    fn menu(gap_before: i32) -> MenuRenderSnapshot {
        MenuRenderSnapshot {
            x: 0,
            y: 10,
            item_w: 100,
            item_h: 8,
            items: vec![MenuItemRenderSnapshot {
                number: 1,
                label: String::new(),
                y_offset: 2,
                gap_before,
            }],
            selected: 0,
            font_color: Rgba::rgb(255, 255, 255),
            box_color: Rgba::rgb(255, 255, 255),
            show_labels: true,
            show_box: true,
        }
    }

    #[test]
    fn menu_item_y_includes_item_offset() {
        assert_eq!(menu_item_y(&menu(0), 0), 12);
    }

    #[test]
    fn menu_item_y_places_normal_item_after_gap() {
        let mut snapshot = menu(0);
        snapshot.items.push(MenuItemRenderSnapshot {
            number: 0,
            label: String::new(),
            y_offset: 0,
            gap_before: 4,
        });
        assert_eq!(menu_item_y(&snapshot, 1), 22);
    }
}
