use crate::oxide::draw::CommandBuffer;
use crate::oxide::draw_renderer::DrawCommandRenderer;
use crate::sprite::SpriteData;
use crate::oxide::Font;
use crate::video::{Renderer, TextureId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    None,
    Texture(TextureId),
}

pub struct OxideRenderer {
    draw_renderer: DrawCommandRenderer,
}

impl Default for OxideRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OxideRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            draw_renderer: DrawCommandRenderer::new(),
        }
    }

    pub fn render_commands(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        commands: &CommandBuffer,
        background: Background,
    ) -> Result<(), String> {
        let texture = match background {
            Background::None => None,
            Background::Texture(texture) => Some(texture),
        };
        self.draw_renderer.render_frame(
            renderer,
            font,
            sprites,
            commands.commands(),
            texture,
        )
    }
}
