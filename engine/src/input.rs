use std::collections::HashSet;
use sdl2::EventPump;

pub struct Input {
    event_pump: EventPump,
    keys_held: HashSet<sdl2::keyboard::Keycode>,
    keys_pressed: Vec<sdl2::keyboard::Keycode>,
    running: bool,
}

impl Input {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = sdl.event_pump().map_err(|e| e.to_string())?;
        Ok(Self { event_pump, keys_held: HashSet::new(), keys_pressed: Vec::new(), running: true })
    }

    pub fn poll(&mut self) {
        use sdl2::event::Event;
        self.keys_pressed.clear();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown { keycode: Some(k), .. } => {
                    self.keys_held.insert(k);
                    self.keys_pressed.push(k);
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    self.keys_held.remove(&k);
                }
                _ => {}
            }
        }
    }

    pub fn last_key(&self) -> Option<sdl2::keyboard::Keycode> {
        self.keys_pressed.last().copied()
    }

    pub fn is_held(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn running(&self) -> bool {
        self.running
    }
}
