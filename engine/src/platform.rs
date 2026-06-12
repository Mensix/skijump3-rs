use crate::input::Input;
use crate::video::Renderer;

/// Owns platform services that must be initialized and dropped together.
///
/// The game crate should depend on normalized engine services, not on SDL.
pub struct Runtime {
    renderer: Renderer,
    input: Input,
    _sdl: sdl2::Sdl,
}

impl Runtime {
    /// Initialize SDL-backed renderer and input services.
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let renderer = Renderer::new(&sdl)?;
        let input = Input::new(&sdl)?;

        Ok(Self {
            renderer,
            input,
            _sdl: sdl,
        })
    }

    #[must_use]
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    #[must_use]
    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }
}
