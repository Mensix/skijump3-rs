use crate::app::router::AppRouter;
use crate::gfx::theme::FONT_HELP;
use engine::oxide::Font;
use engine::oxide::{Background, CommandBuffer, OxideRenderer, PaintCx, ScreenBackground};
use engine::palette::Palette;
use engine::sprite::SpriteData;
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
        palette: &Palette,
        sprites: &[SpriteData],
        router: &AppRouter,
        main_background: TextureId,
    ) -> Result<(), String> {
        let mut commands = CommandBuffer::new();
        {
            let mut cx = PaintCx::new(&mut commands);
            router.paint(&mut cx);
            self.add_debug_overlay(&mut cx);
        }

        let background = match router.screen_background() {
            ScreenBackground::MainPng => Background::Texture(main_background),
            ScreenBackground::NoneBlack => Background::None,
        };

        self.renderer
            .render_commands(renderer, font, palette, sprites, &commands, background)?;
        renderer.wait_frame();
        Ok(())
    }

    fn add_debug_overlay(&mut self, cx: &mut PaintCx<'_>) {
        let fps = self.fps.tick();
        if cfg!(debug_assertions) {
            cx.right_text((319, 192), FONT_HELP, format!("{fps:.0} fps"));
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
