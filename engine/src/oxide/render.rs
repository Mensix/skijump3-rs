use crate::element_renderer::ElementRenderContext;
use crate::oxide::draw::CommandBuffer;
use crate::oxide::legacy::draw_command_to_element;
use crate::sprite::SpriteData;
use crate::ui::{Element, Font};
use crate::video::{Renderer, TextureId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    None,
    Texture(TextureId),
}

pub struct OxideRenderer {
    legacy: ElementRenderContext,
    legacy_elements: Vec<Element>,
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
            legacy: ElementRenderContext::new(),
            legacy_elements: Vec::new(),
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
        self.legacy_elements.clear();
        self.legacy_elements.extend(
            commands
                .commands()
                .iter()
                .cloned()
                .map(draw_command_to_element),
        );
        self.render_legacy_elements(renderer, font, sprites, background)
    }

    pub fn render_legacy(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        elements: Vec<Element>,
        background: Background,
    ) -> Result<(), String> {
        self.legacy_elements = elements;
        self.render_legacy_elements(renderer, font, sprites, background)
    }

    fn render_legacy_elements(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        background: Background,
    ) -> Result<(), String> {
        let texture = match background {
            Background::None => None,
            Background::Texture(texture) => Some(texture),
        };
        self.legacy
            .render_frame(renderer, font, sprites, &self.legacy_elements, texture)
    }
}
