pub use crate::ui::Key;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiEvent {
    KeyDown(Key),
    Text(char),
    Quit,
    Tick,
}

impl From<crate::ui::Event> for UiEvent {
    fn from(event: crate::ui::Event) -> Self {
        match event {
            crate::ui::Event::Keyboard(Key::Char(c)) => Self::Text(c),
            crate::ui::Event::Keyboard(key) => Self::KeyDown(key),
        }
    }
}
