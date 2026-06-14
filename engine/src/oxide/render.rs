use crate::oxide::draw::CommandBuffer;
use crate::oxide::draw_renderer::{DrawCommandRenderer, DrawRenderAssets};
use crate::oxide::Font;
use crate::sprite::BakedSpriteTextures;
use crate::video::{Renderer, TextureId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    None,
    Texture(TextureId),
}

pub struct OxideRenderer {
    draw_renderer: DrawCommandRenderer,
}

pub struct RenderAssets<'a> {
    pub font: &'a Font,
    pub baked_sprites: &'a BakedSpriteTextures,
    pub pattern_texture: TextureId,
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
        assets: RenderAssets<'_>,
        commands: &CommandBuffer,
        background: Background,
    ) -> Result<(), String> {
        let texture = match background {
            Background::None => None,
            Background::Texture(texture) => Some(texture),
        };
        self.draw_renderer.render_frame(
            renderer,
            DrawRenderAssets {
                font: assets.font,
                baked_sprites: assets.baked_sprites,
                pattern_texture: assets.pattern_texture,
            },
            commands.commands(),
            texture,
        )
    }
}
