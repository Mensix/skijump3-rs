use crate::ui::{Event, Key};
use sdl2::EventPump;
use std::collections::HashSet;

pub struct Input {
    event_pump: EventPump,
    keys_held: HashSet<sdl2::keyboard::Keycode>,
    running: bool,
}

impl Input {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = sdl.event_pump()?;
        Ok(Self {
            event_pump,
            keys_held: HashSet::new(),
            running: true,
        })
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        use sdl2::event::Event as SdlEvent;
        use sdl2::keyboard::Keycode;
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                SdlEvent::Quit { .. } => self.running = false,
                SdlEvent::KeyDown {
                    keycode: Some(k),
                    keymod,
                    ..
                } => {
                    self.keys_held.insert(k);
                    let shifted = keymod.intersects(
                        sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD,
                    );
                    let mapped = match k {
                        Keycode::Up => Key::Up,
                        Keycode::Down => Key::Down,
                        Keycode::Left => Key::Left,
                        Keycode::Right => Key::Right,
                        Keycode::Home => Key::Home,
                        Keycode::End => Key::End,
                        Keycode::PageUp => Key::PageUp,
                        Keycode::PageDown => Key::PageDown,
                        Keycode::Return => Key::Enter,
                        Keycode::Escape => Key::Escape,
                        Keycode::Backspace => Key::Backspace,
                        Keycode::Delete => Key::Delete,
                        Keycode::F5 => Key::F5,
                        Keycode::Space => Key::Char(' '),
                        Keycode::Minus | Keycode::KpMinus => Key::Char('-'),
                        Keycode::Equals if shifted => Key::Char('+'),
                        Keycode::KpPlus => Key::Char('+'),
                        Keycode::Period => Key::Char('.'),
                        Keycode::Comma => Key::Char(','),
                        Keycode::Hash => Key::Char('#'),
                        Keycode::Exclaim => Key::Char('!'),
                        Keycode::LeftParen => Key::Char('('),
                        Keycode::RightParen => Key::Char(')'),
                        Keycode::A => Key::Char(if shifted { 'A' } else { 'a' }),
                        Keycode::B => Key::Char(if shifted { 'B' } else { 'b' }),
                        Keycode::C => Key::Char(if shifted { 'C' } else { 'c' }),
                        Keycode::D => Key::Char(if shifted { 'D' } else { 'd' }),
                        Keycode::E => Key::Char(if shifted { 'E' } else { 'e' }),
                        Keycode::F => Key::Char(if shifted { 'F' } else { 'f' }),
                        Keycode::G => Key::Char(if shifted { 'G' } else { 'g' }),
                        Keycode::H => Key::Char(if shifted { 'H' } else { 'h' }),
                        Keycode::I => Key::Char(if shifted { 'I' } else { 'i' }),
                        Keycode::J => Key::Char(if shifted { 'J' } else { 'j' }),
                        Keycode::K => Key::Char(if shifted { 'K' } else { 'k' }),
                        Keycode::L => Key::Char(if shifted { 'L' } else { 'l' }),
                        Keycode::M => Key::Char(if shifted { 'M' } else { 'm' }),
                        Keycode::N => Key::Char(if shifted { 'N' } else { 'n' }),
                        Keycode::O => Key::Char(if shifted { 'O' } else { 'o' }),
                        Keycode::P => Key::Char(if shifted { 'P' } else { 'p' }),
                        Keycode::Q => Key::Char(if shifted { 'Q' } else { 'q' }),
                        Keycode::R => Key::Char(if shifted { 'R' } else { 'r' }),
                        Keycode::S => Key::Char(if shifted { 'S' } else { 's' }),
                        Keycode::T => Key::Char(if shifted { 'T' } else { 't' }),
                        Keycode::U => Key::Char(if shifted { 'U' } else { 'u' }),
                        Keycode::V => Key::Char(if shifted { 'V' } else { 'v' }),
                        Keycode::W => Key::Char(if shifted { 'W' } else { 'w' }),
                        Keycode::X => Key::Char(if shifted { 'X' } else { 'x' }),
                        Keycode::Y => Key::Char(if shifted { 'Y' } else { 'y' }),
                        Keycode::Z => Key::Char(if shifted { 'Z' } else { 'z' }),
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
                SdlEvent::KeyUp {
                    keycode: Some(k), ..
                } => {
                    self.keys_held.remove(&k);
                }
                _ => {}
            }
        }
        events
    }

    #[must_use]
    pub fn is_held(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.keys_held.contains(&key)
    }

    #[must_use]
    pub fn running(&self) -> bool {
        self.running
    }
}
