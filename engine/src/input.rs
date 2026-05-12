use std::collections::HashSet;
use sdl2::EventPump;
use crate::ui::{Event, Key};

pub struct Input {
    event_pump: EventPump,
    keys_held: HashSet<sdl2::keyboard::Keycode>,
    running: bool,
}

impl Input {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = sdl.event_pump().map_err(|e| e.to_string())?;
        Ok(Self { event_pump, keys_held: HashSet::new(), running: true })
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        use sdl2::event::Event as SdlEvent;
        use sdl2::keyboard::Keycode;
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                SdlEvent::Quit { .. } => self.running = false,
                SdlEvent::KeyDown { keycode: Some(k), .. } => {
                    self.keys_held.insert(k);
                    let mapped = match k {
                        Keycode::Up => Key::Up,
                        Keycode::Down => Key::Down,
                        Keycode::Left => Key::Left,
                        Keycode::Right => Key::Right,
                        Keycode::Return => Key::Enter,
                        Keycode::Escape => Key::Escape,
                        Keycode::Num0 => Key::Char('0'),
                        Keycode::Num1 => Key::Char('1'),
                        Keycode::Num2 => Key::Char('2'),
                        Keycode::Num3 => Key::Char('3'),
                        Keycode::Num4 => Key::Char('4'),
                        Keycode::Num5 => Key::Char('5'),
                        Keycode::Num6 => Key::Char('6'),
                        Keycode::Num7 => Key::Char('7'),
                        Keycode::Num8 => Key::Char('8'),
                        Keycode::Num9 => Key::Char('9'),
                        _ => continue,
                    };
                    events.push(Event::Keyboard(mapped));
                }
                SdlEvent::KeyUp { keycode: Some(k), .. } => {
                    self.keys_held.remove(&k);
                }
                _ => {}
            }
        }
        events
    }

    pub fn is_held(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn running(&self) -> bool {
        self.running
    }
}
