use crate::oxide::input::{Key, UiEvent};
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

    pub fn drain_events(&mut self) -> Vec<UiEvent> {
        use sdl2::event::Event as SdlEvent;
        use sdl2::keyboard::Keycode;
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                SdlEvent::Quit { .. } => {
                    self.running = false;
                    events.push(UiEvent::Quit);
                }
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
                        Keycode::Up => Some(Key::Up),
                        Keycode::Down => Some(Key::Down),
                        Keycode::Left => Some(Key::Left),
                        Keycode::Right => Some(Key::Right),
                        Keycode::Home => Some(Key::Home),
                        Keycode::End => Some(Key::End),
                        Keycode::PageUp => Some(Key::PageUp),
                        Keycode::PageDown => Some(Key::PageDown),
                        Keycode::Return => Some(Key::Enter),
                        Keycode::Escape => Some(Key::Escape),
                        Keycode::Backspace => Some(Key::Backspace),
                        Keycode::Delete => Some(Key::Delete),
                        Keycode::Tab => Some(Key::Tab),
                        Keycode::F1 => Some(Key::F1),
                        Keycode::F2 => Some(Key::F2),
                        Keycode::F3 => Some(Key::F3),
                        Keycode::F4 => Some(Key::F4),
                        Keycode::F5 => Some(Key::F5),
                        Keycode::F6 => Some(Key::F6),
                        Keycode::F7 => Some(Key::F7),
                        Keycode::F8 => Some(Key::F8),
                        Keycode::F9 => Some(Key::F9),
                        Keycode::F10 => Some(Key::F10),
                        Keycode::Space => {
                            events.push(UiEvent::Text(' '));
                            None
                        }
                        Keycode::Minus | Keycode::KpMinus => {
                            events.push(UiEvent::Text('-'));
                            None
                        }
                        Keycode::Equals if shifted => {
                            events.push(UiEvent::Text('+'));
                            None
                        }
                        Keycode::KpPlus => {
                            events.push(UiEvent::Text('+'));
                            None
                        }
                        Keycode::Period => {
                            events.push(UiEvent::Text('.'));
                            None
                        }
                        Keycode::Comma => {
                            events.push(UiEvent::Text(','));
                            None
                        }
                        Keycode::Hash => {
                            events.push(UiEvent::Text('#'));
                            None
                        }
                        Keycode::Exclaim => {
                            events.push(UiEvent::Text('!'));
                            None
                        }
                        Keycode::LeftParen => {
                            events.push(UiEvent::Text('('));
                            None
                        }
                        Keycode::RightParen => {
                            events.push(UiEvent::Text(')'));
                            None
                        }
                        Keycode::A => {
                            events.push(UiEvent::Text(if shifted { 'A' } else { 'a' }));
                            None
                        }
                        Keycode::B => {
                            events.push(UiEvent::Text(if shifted { 'B' } else { 'b' }));
                            None
                        }
                        Keycode::C => {
                            events.push(UiEvent::Text(if shifted { 'C' } else { 'c' }));
                            None
                        }
                        Keycode::D => {
                            events.push(UiEvent::Text(if shifted { 'D' } else { 'd' }));
                            None
                        }
                        Keycode::E => {
                            events.push(UiEvent::Text(if shifted { 'E' } else { 'e' }));
                            None
                        }
                        Keycode::F => {
                            events.push(UiEvent::Text(if shifted { 'F' } else { 'f' }));
                            None
                        }
                        Keycode::G => {
                            events.push(UiEvent::Text(if shifted { 'G' } else { 'g' }));
                            None
                        }
                        Keycode::H => {
                            events.push(UiEvent::Text(if shifted { 'H' } else { 'h' }));
                            None
                        }
                        Keycode::I => {
                            events.push(UiEvent::Text(if shifted { 'I' } else { 'i' }));
                            None
                        }
                        Keycode::J => {
                            events.push(UiEvent::Text(if shifted { 'J' } else { 'j' }));
                            None
                        }
                        Keycode::K => {
                            events.push(UiEvent::Text(if shifted { 'K' } else { 'k' }));
                            None
                        }
                        Keycode::L => {
                            events.push(UiEvent::Text(if shifted { 'L' } else { 'l' }));
                            None
                        }
                        Keycode::M => {
                            events.push(UiEvent::Text(if shifted { 'M' } else { 'm' }));
                            None
                        }
                        Keycode::N => {
                            events.push(UiEvent::Text(if shifted { 'N' } else { 'n' }));
                            None
                        }
                        Keycode::O => {
                            events.push(UiEvent::Text(if shifted { 'O' } else { 'o' }));
                            None
                        }
                        Keycode::P => {
                            events.push(UiEvent::Text(if shifted { 'P' } else { 'p' }));
                            None
                        }
                        Keycode::Q => {
                            events.push(UiEvent::Text(if shifted { 'Q' } else { 'q' }));
                            None
                        }
                        Keycode::R => {
                            events.push(UiEvent::Text(if shifted { 'R' } else { 'r' }));
                            None
                        }
                        Keycode::S => {
                            events.push(UiEvent::Text(if shifted { 'S' } else { 's' }));
                            None
                        }
                        Keycode::T => {
                            events.push(UiEvent::Text(if shifted { 'T' } else { 't' }));
                            None
                        }
                        Keycode::U => {
                            events.push(UiEvent::Text(if shifted { 'U' } else { 'u' }));
                            None
                        }
                        Keycode::V => {
                            events.push(UiEvent::Text(if shifted { 'V' } else { 'v' }));
                            None
                        }
                        Keycode::W => {
                            events.push(UiEvent::Text(if shifted { 'W' } else { 'w' }));
                            None
                        }
                        Keycode::X => {
                            events.push(UiEvent::Text(if shifted { 'X' } else { 'x' }));
                            None
                        }
                        Keycode::Y => {
                            events.push(UiEvent::Text(if shifted { 'Y' } else { 'y' }));
                            None
                        }
                        Keycode::Z => {
                            events.push(UiEvent::Text(if shifted { 'Z' } else { 'z' }));
                            None
                        }
                        Keycode::Num0 => {
                            events.push(UiEvent::Text('0'));
                            None
                        }
                        Keycode::Num1 => {
                            events.push(UiEvent::Text('1'));
                            None
                        }
                        Keycode::Num2 => {
                            events.push(UiEvent::Text('2'));
                            None
                        }
                        Keycode::Num3 => {
                            events.push(UiEvent::Text('3'));
                            None
                        }
                        Keycode::Num4 => {
                            events.push(UiEvent::Text('4'));
                            None
                        }
                        Keycode::Num5 => {
                            events.push(UiEvent::Text('5'));
                            None
                        }
                        Keycode::Num6 => {
                            events.push(UiEvent::Text('6'));
                            None
                        }
                        Keycode::Num7 => {
                            events.push(UiEvent::Text('7'));
                            None
                        }
                        Keycode::Num8 => {
                            events.push(UiEvent::Text('8'));
                            None
                        }
                        Keycode::Num9 => {
                            events.push(UiEvent::Text('9'));
                            None
                        }
                        _ => None,
                    };
                    if let Some(mapped) = mapped {
                        events.push(UiEvent::KeyDown(mapped));
                    }
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
