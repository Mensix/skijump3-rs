use crate::jump::{JumpInput, JumpPhase, JumpSession};
use crate::save::config::Config;
use engine::oxide::input::{Key, UiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpInputAction {
    None,
    RouteBack,
    ResetWind,
    ResetJump,
    PersistStartGate(i32),
    SaveReplay,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JumpInputController;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JumpKeyBindings {
    up: i32,
    right: i32,
    left: i32,
    telemark: i32,
    replay: i32,
}

impl JumpKeyBindings {
    pub(crate) fn from_config(config: &Config) -> Self {
        Self {
            up: config.key_up,
            right: config.key_right,
            left: config.key_left,
            telemark: config.key_telemark,
            replay: config.key_replay,
        }
    }

    pub(crate) fn code_for(event: UiEvent) -> Option<i32> {
        match event {
            UiEvent::KeyDown(Key::Up) => Some(72),
            UiEvent::KeyDown(Key::Right) => Some(77),
            UiEvent::KeyDown(Key::Left) => Some(75),
            UiEvent::KeyDown(Key::Down) => Some(80),
            UiEvent::KeyDown(Key::Home) => Some(71),
            UiEvent::KeyDown(Key::End) => Some(79),
            UiEvent::KeyDown(Key::PageUp) => Some(73),
            UiEvent::KeyDown(Key::PageDown) => Some(81),
            UiEvent::KeyDown(Key::Backspace) => Some(8 << 8),
            UiEvent::KeyDown(Key::Delete) => Some(83),
            UiEvent::Text(c) => char_key_code(c),
            _ => None,
        }
    }

    fn matches_up(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.up)
    }

    fn matches_right(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.right)
    }

    fn matches_left(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.left)
    }

    fn matches_telemark(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.telemark)
    }

    fn matches_replay(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.replay)
    }
}

pub(crate) fn char_key_code(c: char) -> Option<i32> {
    let c = c.to_ascii_uppercase();
    let scan = match c {
        'A' => 30,
        'B' => 48,
        'C' => 46,
        'D' => 32,
        'E' => 18,
        'F' => 33,
        'G' => 34,
        'H' => 35,
        'I' => 23,
        'J' => 36,
        'K' => 37,
        'L' => 38,
        'M' => 50,
        'N' => 49,
        'O' => 24,
        'P' => 25,
        'Q' => 16,
        'R' => 19,
        'S' => 31,
        'T' => 20,
        'U' => 22,
        'V' => 47,
        'W' => 17,
        'X' => 45,
        'Y' => 21,
        'Z' => 44,
        '0' => 11,
        '1' => 2,
        '2' => 3,
        '3' => 4,
        '4' => 5,
        '5' => 6,
        '6' => 7,
        '7' => 8,
        '8' => 9,
        '9' => 10,
        ' ' => 57,
        '.' => 52,
        ',' => 51,
        '-' => 12,
        '+' => 13,
        '/' => 53,
        '*' => 55,
        _ => return None,
    };
    Some((c as i32) * 256 + scan)
}

impl JumpInputController {
    pub(crate) fn handle_event(
        self,
        event: UiEvent,
        session: &mut JumpSession,
        keys: JumpKeyBindings,
    ) -> JumpInputAction {
        match event {
            UiEvent::KeyDown(Key::Escape) => JumpInputAction::RouteBack,
            UiEvent::KeyDown(Key::F5) => {
                if session.policy().allow_wind_reset {
                    JumpInputAction::ResetWind
                } else {
                    JumpInputAction::None
                }
            }
            UiEvent::KeyDown(Key::Enter) => Self::enter(session),
            UiEvent::Text('+') => Self::adjust_gate(session, 1),
            UiEvent::Text('-') => Self::adjust_gate(session, -1),
            _ if keys.matches_right(event) => Self::right(session),
            _ if keys.matches_left(event) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::LeanBack);
                }
                JumpInputAction::None
            }
            _ if keys.matches_up(event) => {
                if session.phase() == Some(JumpPhase::Inrun) {
                    session.handle_input(JumpInput::Takeoff);
                }
                JumpInputAction::None
            }
            _ if keys.matches_telemark(event) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::Telemark);
                }
                JumpInputAction::None
            }
            _ if keys.matches_replay(event) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::TwoFooted);
                }
                JumpInputAction::None
            }
            UiEvent::Text('s' | 'S') => {
                if session.phase() == Some(JumpPhase::Result) {
                    JumpInputAction::SaveReplay
                } else {
                    JumpInputAction::None
                }
            }
            _ => JumpInputAction::None,
        }
    }

    fn enter(session: &mut JumpSession) -> JumpInputAction {
        match session.phase() {
            Some(JumpPhase::Result) => JumpInputAction::ResetJump,
            Some(JumpPhase::Info) => {
                let start_gate = session.start_gate().unwrap_or_default();
                session.handle_input(JumpInput::LeaveInfo);
                JumpInputAction::PersistStartGate(start_gate)
            }
            Some(JumpPhase::Landing) => {
                session.handle_input(JumpInput::ShowResult);
                JumpInputAction::None
            }
            _ => {
                session.handle_input(JumpInput::Start);
                JumpInputAction::None
            }
        }
    }

    fn right(session: &mut JumpSession) -> JumpInputAction {
        match session.phase() {
            Some(JumpPhase::Info) => {
                let start_gate = session.start_gate().unwrap_or_default();
                session.handle_input(JumpInput::LeaveInfo);
                JumpInputAction::PersistStartGate(start_gate)
            }
            Some(JumpPhase::OnBar) => {
                session.handle_input(JumpInput::Start);
                JumpInputAction::None
            }
            _ => {
                session.handle_input(JumpInput::LeanForward);
                JumpInputAction::None
            }
        }
    }

    fn adjust_gate(session: &mut JumpSession, delta: i32) -> JumpInputAction {
        if session.phase() != Some(JumpPhase::Info) {
            return JumpInputAction::None;
        }
        session
            .handle_start_gate_adjust(delta)
            .map_or(JumpInputAction::None, JumpInputAction::PersistStartGate)
    }
}
