use crate::audio::TonePlayer;
use crate::input::Input;
use crate::video::Renderer;

pub struct Context {
    _sdl: sdl3::Sdl,
    pub audio: TonePlayer,
    pub renderer: Renderer,
    pub input: Input,
}

impl Context {
    pub fn new(software_rendering: bool) -> Result<Self, String> {
        let sdl = sdl3::init().map_err(|error| format!("SDL init failed: {error}"))?;
        let audio = TonePlayer::new(&sdl);
        let renderer = Renderer::new(&sdl, software_rendering)?;
        let input = Input::new(&sdl);
        Ok(Self {
            _sdl: sdl,
            audio,
            renderer,
            input,
        })
    }
}
