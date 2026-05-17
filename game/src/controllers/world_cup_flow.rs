use crate::competition::machine::{Competition, StepDecision};
use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::types::JumpOutcome;
use std::cell::Cell;

pub enum WorldCupCommand {
    HumanJump {
        participant: JumpParticipant,
        hill_idx: usize,
        phase: CompetitionPhase,
        is_new_event: bool,
    },
    ShowResults,
    Done,
}

/// Convert a competition Participant to a jump-domain JumpParticipant.
/// Lives here (the boundary) so neither `jump` nor `competition` needs
/// to know about the other.
pub(crate) fn participant_to_jump(p: &Participant) -> JumpParticipant {
    JumpParticipant {
        id: p.id,
        ai_id: p.ai_id,
        name: p.name.clone(),
        real_name: p.real_name.clone(),
        suit_color: p.suit_color,
        ski_color: p.ski_color,
        team: p.team,
        control: if p.is_computer {
            JumperControl::Computer
        } else {
            JumperControl::Human
        },
    }
}

/// Advance competition state until the next user-visible moment.
/// Pure with respect to store/resources — takes `&mut Competition` directly.
pub(crate) fn drive(
    competition: &mut Competition,
    last_event: &Cell<usize>,
    simulate_computer: &mut dyn FnMut(JumpParticipant, usize) -> JumpOutcome,
) -> WorldCupCommand {
    loop {
        match competition.decide_next() {
            StepDecision::Done => return WorldCupCommand::Done,
            StepDecision::ShowResults => return WorldCupCommand::ShowResults,
            StepDecision::AdvancePhase => {
                competition.advance();
                continue;
            }
            StepDecision::Jump {
                idx,
                hill_idx,
                is_human: true,
            } => {
                let participant = participant_to_jump(competition.field.get(idx));
                let is_new_event = competition.current_event != last_event.get();
                if is_new_event {
                    last_event.set(competition.current_event);
                }
                return WorldCupCommand::HumanJump {
                    participant,
                    hill_idx,
                    phase: competition.phase,
                    is_new_event,
                };
            }
            StepDecision::Jump {
                idx,
                hill_idx,
                is_human: false,
            } => {
                let participant = participant_to_jump(competition.field.get(idx));
                let outcome = simulate_computer(participant, hill_idx);

                competition.record_jump(outcome.score, outcome.distance);
                competition.advance();

                if competition.is_over() {
                    return WorldCupCommand::Done;
                }

                if let Some(next_idx) = competition.current_jumper() {
                    if !competition.field.get(next_idx).is_computer {
                        let next_participant = participant_to_jump(competition.field.get(next_idx));
                        let is_new_event = competition.current_event != last_event.get();
                        if is_new_event {
                            last_event.set(competition.current_event);
                        }
                        return WorldCupCommand::HumanJump {
                            participant: next_participant,
                            hill_idx,
                            phase: competition.phase,
                            is_new_event,
                        };
                    }
                }
            }
        }
    }
}
