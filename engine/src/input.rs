use crate::oxide::input::{Key, Modifiers, UiEvent};
use sdl3::EventPump;

pub struct Input {
    event_pump: Option<EventPump>,
    running: bool,
}

impl Input {
    pub fn new(sdl: &sdl3::Sdl) -> Self {
        let event_pump = sdl.event_pump().ok();
        Self {
            event_pump,
            running: true,
        }
    }

    pub fn drain_events(&mut self) -> Vec<UiEvent> {
        use sdl3::event::Event as SdlEvent;
        let mut events = Vec::new();
        let Some(event_pump) = self.event_pump.as_mut() else {
            self.running = false;
            return events;
        };
        for event in event_pump.poll_iter() {
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
                    let shifted = keymod.intersects(
                        sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD,
                    );
                    let modifiers = Modifiers {
                        ctrl: keymod.intersects(
                            sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::RCTRLMOD,
                        ),
                        alt: keymod.intersects(
                            sdl3::keyboard::Mod::LALTMOD | sdl3::keyboard::Mod::RALTMOD,
                        ),
                    };
                    if let Some(character) = key_text(k, shifted) {
                        events.push(UiEvent::Text(character));
                    }
                    if modifiers.ctrl || modifiers.alt {
                        if let Some(UiEvent::Text(character)) = events.last().copied() {
                            events.pop();
                            events.push(UiEvent::TextWithModifiers(character, modifiers));
                        }
                    }
                    if let Some(mapped) = map_special_key(k) {
                        events.push(UiEvent::KeyDown(mapped));
                    }
                }
                _ => {}
            }
        }
        events
    }

    pub fn running(&self) -> bool {
        self.running
    }
}

fn map_special_key(k: sdl3::keyboard::Keycode) -> Option<Key> {
    use sdl3::keyboard::Keycode;
    Some(match k {
        Keycode::Up => Key::Up,
        Keycode::Down => Key::Down,
        Keycode::Left => Key::Left,
        Keycode::Right => Key::Right,
        Keycode::Home => Key::Home,
        Keycode::End => Key::End,
        Keycode::Insert => Key::Insert,
        Keycode::PageUp => Key::PageUp,
        Keycode::PageDown => Key::PageDown,
        Keycode::Kp5 => Key::Kp5,
        Keycode::Return => Key::Enter,
        Keycode::Escape => Key::Escape,
        Keycode::Backspace => Key::Backspace,
        Keycode::Delete => Key::Delete,
        Keycode::Tab => Key::Tab,
        Keycode::F1 => Key::F1,
        Keycode::F2 => Key::F2,
        Keycode::F3 => Key::F3,
        Keycode::F4 => Key::F4,
        Keycode::F5 => Key::F5,
        Keycode::F6 => Key::F6,
        Keycode::F7 => Key::F7,
        Keycode::F8 => Key::F8,
        Keycode::F9 => Key::F9,
        Keycode::F10 => Key::F10,
        _ => return None,
    })
}

fn key_text(k: sdl3::keyboard::Keycode, shifted: bool) -> Option<char> {
    use sdl3::keyboard::Keycode;
    match k {
        Keycode::Space => Some(' '),
        Keycode::Minus | Keycode::KpMinus => Some('-'),
        Keycode::Equals if shifted => Some('+'),
        Keycode::Plus | Keycode::KpPlus => Some('+'),
        Keycode::Slash | Keycode::KpDivide => Some('/'),
        Keycode::Asterisk | Keycode::KpMultiply => Some('*'),
        Keycode::Period => Some('.'),
        Keycode::Comma => Some(','),
        Keycode::Hash => Some('#'),
        Keycode::Exclaim => Some('!'),
        Keycode::LeftParen => Some('('),
        Keycode::RightParen => Some(')'),
        Keycode::_0 => Some('0'),
        Keycode::_1 => Some('1'),
        Keycode::_2 => Some('2'),
        Keycode::_3 => Some('3'),
        Keycode::_4 => Some('4'),
        Keycode::_5 => Some('5'),
        Keycode::_6 => Some('6'),
        Keycode::_7 => Some('7'),
        Keycode::_8 => Some(if shifted { '*' } else { '8' }),
        Keycode::_9 => Some('9'),
        k if (Keycode::A as i32..=Keycode::Z as i32).contains(&(k as i32)) => {
            let base = (k as i32 - Keycode::A as i32) as u8;
            Some((if shifted { b'A' } else { b'a' } + base) as char)
        }
        _ => None,
    }
}
