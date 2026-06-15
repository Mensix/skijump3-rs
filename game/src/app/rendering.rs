use crate::app::router::AppRouter;
use crate::gfx::theme::FONT_GRAY;
use engine::oxide::{
    Background, CommandBuffer, Font, OxideRenderer, PaintCx, RenderAssets, ScreenBackground,
};
use engine::sprite::BakedSpriteTextures;
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
        assets: FrameAssets<'_>,
        router: &AppRouter,
    ) -> Result<(), String> {
        let mut commands = CommandBuffer::new();
        {
            let mut cx = PaintCx::new(&mut commands);
            router.paint(&mut cx);
            self.add_debug_overlay(&mut cx);
        }

        let background = match router.screen_background() {
            ScreenBackground::MainPng => Background::Texture(assets.main_background),
            ScreenBackground::NoneBlack => Background::None,
        };

        self.renderer.render_commands(
            renderer,
            RenderAssets {
                font: assets.font,
                baked_sprites: assets.baked_sprites,
                pattern_texture: assets.pattern_texture,
            },
            &commands,
            background,
        )?;
        renderer.wait_frame();
        Ok(())
    }

    fn add_debug_overlay(&mut self, cx: &mut PaintCx<'_>) {
        let fps = self.fps.tick();
        if cfg!(debug_assertions) {
            cx.right_text((319, 192), FONT_GRAY, format!("{fps:.0} fps"));
        }
    }
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
