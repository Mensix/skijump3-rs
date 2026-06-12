#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Keyboard(Key),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Escape,
    Backspace,
    Delete,
    F5,
    Char(char),
}
