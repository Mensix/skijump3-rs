use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::jump::config::JumpParticipant;
use crate::jump::types::JumpOutcome;

#[derive(Debug)]
pub(crate) enum CompetitionFlowCommand<C, R> {
    HumanJump {
        participant: JumpParticipant,
        hill_idx: usize,
        context: C,
        is_new_event: bool,
    },
    ShowResults(R),
    Done,
}

pub(crate) fn drive<R, E>(
    runtime: &mut R,
    simulate_computer: &mut dyn FnMut(JumpParticipant, usize) -> Result<JumpOutcome, E>,
    mark_new_event: &mut dyn FnMut(&R::Context, bool) -> bool,
) -> Result<CompetitionFlowCommand<R::Context, R::ResultsKind>, E>
where
    R: CompetitionRuntime,
{
    loop {
        match runtime.decide_next_runtime() {
            CompetitionDecision::ShowResults(kind) => {
                return Ok(CompetitionFlowCommand::ShowResults(kind));
            }
            CompetitionDecision::Done => return Ok(CompetitionFlowCommand::Done),
            CompetitionDecision::Jump {
                participant,
                hill_idx,
                context,
                is_human,
                is_new_event,
            } => {
                if is_human {
                    return Ok(CompetitionFlowCommand::HumanJump {
                        participant,
                        hill_idx,
                        is_new_event: mark_new_event(&context, is_new_event),
                        context,
                    });
                }

                let outcome = simulate_computer(participant, hill_idx)?;
                runtime.record_jump_runtime(&context, outcome);
                if runtime.is_complete_runtime() {
                    return Ok(CompetitionFlowCommand::Done);
                }
            }
        }
    }
}
