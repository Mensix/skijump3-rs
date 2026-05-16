use crate::jump::{JumpInput, JumpPhase, JumpSession};
use engine::ui::{Event, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrainingJumpAction {
    None,
    RoutePractice,
    ResetWind,
    ResetJump,
    PersistStartGate(i32),
    SaveReplay,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrainingJumpController;

impl TrainingJumpController {
    pub(crate) fn handle_event(
        self,
        event: Event,
        session: &mut JumpSession,
    ) -> TrainingJumpAction {
        match event {
            Event::Keyboard(Key::Escape) => TrainingJumpAction::RoutePractice,
            Event::Keyboard(Key::F5) => {
                if session.policy().allow_wind_reset {
                    TrainingJumpAction::ResetWind
                } else {
                    TrainingJumpAction::None
                }
            }
            Event::Keyboard(Key::Enter) => self.enter(session),
            Event::Keyboard(Key::Right) => self.right(session),
            Event::Keyboard(Key::Char('+')) => self.adjust_gate(session, 1),
            Event::Keyboard(Key::Char('-')) => self.adjust_gate(session, -1),
            Event::Keyboard(Key::Left) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::LeanBack);
                }
                TrainingJumpAction::None
            }
            Event::Keyboard(Key::Up) => {
                if session.phase() == Some(JumpPhase::Inrun) {
                    session.handle_input(JumpInput::Takeoff);
                }
                TrainingJumpAction::None
            }
            Event::Keyboard(Key::Char('t' | 'T')) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::Telemark);
                }
                TrainingJumpAction::None
            }
            Event::Keyboard(Key::Char('r' | 'R')) => {
                if session.phase() == Some(JumpPhase::Flight) {
                    session.handle_input(JumpInput::TwoFooted);
                }
                TrainingJumpAction::None
            }
            Event::Keyboard(Key::Char('s' | 'S')) => {
                if session.phase() == Some(JumpPhase::Result) {
                    TrainingJumpAction::SaveReplay
                } else {
                    TrainingJumpAction::None
                }
            }
            _ => TrainingJumpAction::None,
        }
    }

    fn enter(self, session: &mut JumpSession) -> TrainingJumpAction {
        match session.phase() {
            Some(JumpPhase::Result) => TrainingJumpAction::ResetJump,
            Some(JumpPhase::Info) => {
                let start_gate = session.start_gate().unwrap_or_default();
                session.handle_input(JumpInput::LeaveInfo);
                TrainingJumpAction::PersistStartGate(start_gate)
            }
            Some(JumpPhase::Landing) => {
                session.handle_input(JumpInput::ShowResult);
                TrainingJumpAction::None
            }
            _ => {
                session.handle_input(JumpInput::Start);
                TrainingJumpAction::None
            }
        }
    }

    fn right(self, session: &mut JumpSession) -> TrainingJumpAction {
        match session.phase() {
            Some(JumpPhase::Info) => {
                let start_gate = session.start_gate().unwrap_or_default();
                session.handle_input(JumpInput::LeaveInfo);
                TrainingJumpAction::PersistStartGate(start_gate)
            }
            Some(JumpPhase::OnBar) => {
                session.handle_input(JumpInput::Start);
                TrainingJumpAction::None
            }
            _ => {
                session.handle_input(JumpInput::LeanForward);
                TrainingJumpAction::None
            }
        }
    }

    fn adjust_gate(self, session: &mut JumpSession, delta: i32) -> TrainingJumpAction {
        if session.phase() != Some(JumpPhase::Info) {
            return TrainingJumpAction::None;
        }
        session.handle_start_gate_adjust(delta).map_or(
            TrainingJumpAction::None,
            TrainingJumpAction::PersistStartGate,
        )
    }
}
