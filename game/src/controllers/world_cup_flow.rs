use crate::competition::machine::{Competition, StepDecision};
use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::types::JumpOutcome;
use std::cell::Cell;

#[derive(Debug)]
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

/// Extract event-change check for testability.
fn check_event_change(current_event: usize, last_event: &Cell<usize>) -> bool {
    let new = current_event != last_event.get();
    if new {
        last_event.set(current_event);
    }
    new
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
                let participant = participant_to_jump(competition.participant(idx));
                return WorldCupCommand::HumanJump {
                    participant,
                    hill_idx,
                    phase: competition.phase(),
                    is_new_event: check_event_change(competition.current_event, last_event),
                };
            }
            StepDecision::Jump {
                idx,
                hill_idx,
                is_human: false,
            } => {
                let participant = participant_to_jump(competition.participant(idx));
                let outcome = simulate_computer(participant, hill_idx);

                competition.record_jump(outcome.score, outcome.distance);
                competition.advance();

                if competition.is_over() {
                    return WorldCupCommand::Done;
                }

                if let Some(next_idx) = competition.current_jumper() {
                    if !competition.participant(next_idx).is_computer {
                        let next_participant =
                            participant_to_jump(competition.participant(next_idx));
                        return WorldCupCommand::HumanJump {
                            participant: next_participant,
                            hill_idx,
                            phase: competition.phase(),
                            is_new_event: check_event_change(competition.current_event, last_event),
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::types::{CupStyle, Participant, QualificationStatus};

    fn human_participant(id: usize) -> Participant {
        Participant {
            id,
            ai_id: 0,
            name: format!("Human {id}"),
            real_name: String::new(),
            suit_color: 0,
            ski_color: 0,
            team: None,
            is_computer: false,
            wc_points: 0,
            four_hills_points: 0,
            injury: 0,
            points: 0,
            rank: 0,
            qual: QualificationStatus::NotQualified,
            round1_len: 0,
            round2_len: 0,
            qual_len: 0,
        }
    }

    fn make_competition(num_computers: usize, num_humans: usize, hills: Vec<usize>) -> Competition {
        let total = num_computers + num_humans;
        let mut participants = Vec::with_capacity(total);
        for i in 0..num_computers {
            participants.push(Participant::computer(i, i, format!("CPU {i}")));
        }
        for i in 0..num_humans {
            participants.push(human_participant(num_computers + i));
        }
        let mut c = Competition::new(CupStyle::WorldCup, participants, hills);
        c.trainrounds = 0;
        c
    }

    #[test]
    fn drive_returns_human_jump_for_current_human() {
        let mut c = make_competition(65, 10, vec![0]);
        c.advance();

        let jumper_before = c.current_jumper().unwrap();
        assert!(
            !c.field.get(jumper_before).is_computer,
            "first jumper should be human"
        );
        assert_eq!(c.field.get(jumper_before).points, 0);

        let last_event = Cell::new(0);
        let mut simulate = |_: JumpParticipant, _: usize| -> JumpOutcome {
            panic!("should not be called for human");
        };

        match drive(&mut c, &last_event, &mut simulate) {
            WorldCupCommand::HumanJump {
                participant,
                hill_idx,
                phase: _,
                is_new_event,
            } => {
                assert_eq!(participant.id, jumper_before);
                assert_eq!(hill_idx, 0);
                assert!(!is_new_event, "first event should not be new");
                assert!(
                    !c.field.get(jumper_before).is_computer,
                    "still current jumper"
                );
                assert_eq!(
                    c.field.get(jumper_before).points,
                    0,
                    "should not be implicitly recorded"
                );
            }
            other => panic!("expected HumanJump, got {other:?}"),
        }
    }

    #[test]
    fn drive_does_not_implicitly_record_human_jump() {
        let mut c = make_competition(65, 10, vec![0]);
        c.advance();

        let last_event = Cell::new(0);
        let mut simulate = |_: JumpParticipant, _: usize| -> JumpOutcome {
            panic!("should not be called");
        };

        let jumper = c.current_jumper().unwrap();
        let _ = drive(&mut c, &last_event, &mut simulate);
        assert_eq!(
            c.field.get(jumper).points,
            0,
            "points unchanged after first HumanJump"
        );

        let _ = drive(&mut c, &last_event, &mut simulate);
        assert_eq!(
            c.field.get(jumper).points,
            0,
            "points still unchanged after second HumanJump"
        );
    }

    #[test]
    fn drive_after_manual_record_advances_to_next_state() {
        let mut c = make_competition(65, 10, vec![0]);
        c.advance();

        let last_event = Cell::new(0);
        let mut simulate = |_: JumpParticipant, _: usize| -> JumpOutcome {
            panic!("should not be called");
        };

        // Emulate what record_finished_human_jump + advance does
        // after acknowledgement
        c.record_jump(100, 900);
        c.advance();

        // Drive should now skip simulated computers until the next
        // visible state (HumanJump for next human, or ShowResults)
        let result = drive(&mut c, &last_event, &mut simulate);
        match result {
            WorldCupCommand::HumanJump { .. } => { /* next human is up */ }
            WorldCupCommand::ShowResults => { /* phase is done */ }
            WorldCupCommand::Done => { /* season over */ }
        }
    }

    #[test]
    fn check_event_change_tracks_first_event() {
        let last_event = Cell::new(0);
        assert!(!check_event_change(0, &last_event));
        assert_eq!(last_event.get(), 0, "unchanged when current==last");
    }

    #[test]
    fn check_event_change_detects_new_event() {
        let last_event = Cell::new(0);
        assert!(check_event_change(1, &last_event));
        assert_eq!(last_event.get(), 1, "updated to current");
    }

    #[test]
    fn check_event_change_stays_true_only_once() {
        let last_event = Cell::new(0);
        assert!(check_event_change(1, &last_event));
        assert!(!check_event_change(1, &last_event), "already tracked");
    }
}
