use sdl2::EventPump;

pub struct Input {
    event_pump: EventPump,
    last_key: Option<sdl2::keyboard::Keycode>,
    running: bool,
}

impl Input {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = sdl.event_pump().map_err(|e| e.to_string())?;
        Ok(Self { event_pump, last_key: None, running: true })
    }

    pub fn poll(&mut self) {
        use sdl2::event::Event;
        self.last_key = None;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown { keycode: Some(k), .. } => {
                    self.last_key = Some(k);
                }
                _ => {}
            }
        }
    }

    pub fn last_key(&self) -> Option<sdl2::keyboard::Keycode> {
        self.last_key
    }

    pub fn running(&self) -> bool {
        self.running
    }
}
