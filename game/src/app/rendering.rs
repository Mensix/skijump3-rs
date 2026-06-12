use crate::gfx::palette::FONT_HELP;
use crate::route::RouteTarget;
use engine::oxide::{Background, OxideRenderer};
use engine::sprite::SpriteData;
use engine::ui::{BackgroundMode, Element, Font, Router};
use engine::video::{Renderer, TextureId};
use std::time::Instant;

pub(super) struct FrameRenderer {
    renderer: OxideRenderer,
    fps: FpsCounter,
}

impl FrameRenderer {
    pub(super) fn new() -> Self {
        Self {
            renderer: OxideRenderer::new(),
            fps: FpsCounter::new(),
        }
    }

    pub(super) fn render(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        router: &Router<RouteTarget>,
        main_background: TextureId,
    ) -> Result<(), String> {
        let mut elements = router.current_view().elements();
        self.add_debug_overlay(&mut elements);

        let background = match router.current_view().gpu_background() {
            BackgroundMode::MainPng => Background::Texture(main_background),
            BackgroundMode::NoneBlack => Background::None,
        };

        self.renderer
            .render_legacy(renderer, font, sprites, elements, background)?;
        renderer.wait_frame();
        Ok(())
    }

    fn add_debug_overlay(&mut self, elements: &mut Vec<Element>) {
        let fps = self.fps.tick();
        if cfg!(debug_assertions) {
            elements.push(Element::right_text(
                format!("{fps:.0} fps"),
                319,
                192,
                FONT_HELP,
            ));
        }
    }
}

struct FpsCounter {
    frame_count: u64,
    elapsed: f64,
    display: f64,
    last: Instant,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed: 0.0,
            display: 0.0,
            last: Instant::now(),
        }
    }

    fn tick(&mut self) -> f64 {
        self.frame_count += 1;
        self.elapsed += self.last.elapsed().as_secs_f64();
        self.last = Instant::now();

        if self.elapsed >= 0.5 {
            self.display = self.frame_count as f64 / self.elapsed;
            self.frame_count = 0;
            self.elapsed = 0.0;
        }

        self.display
    }
}
