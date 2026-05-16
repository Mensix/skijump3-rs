use crate::jump::types::{JumpInput, JumpPhase, JumpSnapshot};
use crate::pascal_random::PascalRandom;

pub(crate) trait JumpInputProvider {
    fn inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut PascalRandom) -> Vec<JumpInput>;
}

#[derive(Debug)]
pub(crate) struct ComputerInputProvider {
    started: bool,
}

impl ComputerInputProvider {
    pub(crate) fn new() -> Self {
        Self { started: false }
    }
}

impl JumpInputProvider for ComputerInputProvider {
    fn inputs(&mut self, snapshot: &JumpSnapshot, _rng: &mut PascalRandom) -> Vec<JumpInput> {
        match snapshot.phase {
            JumpPhase::Info => vec![JumpInput::LeaveInfo],
            JumpPhase::OnBar if !self.started => {
                self.started = true;
                vec![JumpInput::Start]
            }
            _ => Vec::new(),
        }
    }
}
