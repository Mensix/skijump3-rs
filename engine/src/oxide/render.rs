use crate::oxide::draw::CommandBuffer;
use crate::oxide::draw_renderer::{DrawCommandRenderer, DrawRenderAssets};
use crate::video::{Renderer, TextureId};

pub struct OxideRenderer {
    draw_renderer: DrawCommandRenderer,
}

impl Default for OxideRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OxideRenderer {
    pub fn new() -> Self {
        Self {
            draw_renderer: DrawCommandRenderer::new(),
        }
    }

    pub fn render_commands(
        &mut self,
        renderer: &mut Renderer,
        assets: DrawRenderAssets<'_>,
        commands: &CommandBuffer,
        background: Option<TextureId>,
    ) {
        self.draw_renderer
            .render_frame(renderer, assets, commands.commands(), background);
    }
}
