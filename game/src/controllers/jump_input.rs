use crate::jump::{JumpInput, JumpPhase, JumpSession};
use engine::ui::{Event, Key};

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

impl JumpInputController {
    pub(crate) fn handle_event(self, event: Event, session: &mut JumpSession) -> JumpInputAction {
        match event {
            Event::Keyboard(Key::Escape) => JumpInputAction::RouteBack,
            Event::Keyboard(Key::F5) => {
                if session.policy().allow_wind_reset {
                    JumpInputAction::ResetWind
                } else {
                    JumpInputAction::None
                }
            }
            Event::Keyboard(Key::Enter) => Self::enter(session),
            Event::Keyboard(Key::Right) => Self::right(session),
            Event::Keyboard(Key::Char('+')) => Self::adjust_gate(session, 1),
            Event::Keyboard(Key::Char('-')) => Self::adjust_gate(session, -1),
            Event::Keyboard(Key::Left) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::LeanBack);
                }
                JumpInputAction::None
            }
            Event::Keyboard(Key::Up) => {
                if session.phase() == Some(JumpPhase::Inrun) {
                    session.handle_input(JumpInput::Takeoff);
                }
                JumpInputAction::None
            }
            Event::Keyboard(Key::Char('t' | 'T')) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::Telemark);
                }
                JumpInputAction::None
            }
            Event::Keyboard(Key::Char('r' | 'R')) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::TwoFooted);
                }
                JumpInputAction::None
            }
            Event::Keyboard(Key::Char('s' | 'S')) => {
                if session.phase() == Some(JumpPhase::Result) {
                    JumpInputAction::SaveReplay
                } else {
                    JumpInputAction::None
                }
            }
            Event::Keyboard(_) => JumpInputAction::None,
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
