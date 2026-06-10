use crate::competition::machine::Competition;
use crate::competition::runtime::{IndividualJumpContext, IndividualResultsKind};
use crate::competition::types::CompetitionPhase;
use crate::jump::config::JumpParticipant;
use crate::jump::types::JumpOutcome;
use crate::views::jump::competition::flow as competition_flow;
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
/// The `simulate_computer` closure may return an error; if it does, `drive`
/// propagates it immediately.
pub(crate) fn drive<E>(
    competition: &mut Competition,
    last_event: &Cell<usize>,
    simulate_computer: &mut dyn FnMut(JumpParticipant, usize) -> Result<JumpOutcome, E>,
) -> Result<WorldCupCommand, E> {
    let mut mark_new_event = |ctx: &IndividualJumpContext, _runtime_flag: bool| {
        check_event_change(ctx.event_idx, last_event)
    };
    let command = competition_flow::drive(competition, simulate_computer, &mut mark_new_event)?;
    Ok(match command {
        competition_flow::CompetitionFlowCommand::HumanJump {
            participant,
            hill_idx,
            context,
            is_new_event,
        } => WorldCupCommand::HumanJump {
            participant,
            hill_idx,
            phase: context.phase,
            is_new_event,
        },
        competition_flow::CompetitionFlowCommand::ShowResults(IndividualResultsKind::Results) => {
            WorldCupCommand::ShowResults
        }
        competition_flow::CompetitionFlowCommand::Done => WorldCupCommand::Done,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::types::{
        CompetitionJumpOutcome, CupStyle, Participant, QualificationStatus,
    };
    use crate::jump::types::FallType;

    fn human_participant(id: usize) -> Participant {
        Participant {
            id,
            ai_id: 0,
            profile_idx: None,
            name: format!("Human {id}"),
            real_name: String::new(),
            suit_color: 0,
            ski_color: 0,
            team: None,
            is_computer: false,
            skip_quali: 0,
            wc_points: 0,
            four_hills_points: 0.0,
            tc_points: 0,
            injury: 0,
            points: None,
            rank: 0,
            round1_rank: 0,
            qual: QualificationStatus::NotQualified,
            round1_len: 0.0,
            round1_score: 0.0,
            round2_len: 0.0,
            qual_len: 0.0,
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
        assert_eq!(c.field.get(jumper_before).points, None);

        let last_event = Cell::new(0);
        let mut simulate = |_: JumpParticipant, _: usize| -> Result<JumpOutcome, &'static str> {
            panic!("should not be called for human");
        };

        match drive(&mut c, &last_event, &mut simulate).unwrap() {
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
                    None,
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
        let mut simulate = |_: JumpParticipant, _: usize| -> Result<JumpOutcome, &'static str> {
            panic!("should not be called");
        };

        let jumper = c.current_jumper().unwrap();
        let _ = drive(&mut c, &last_event, &mut simulate).unwrap();
        assert_eq!(
            c.field.get(jumper).points,
            None,
            "points unchanged after first HumanJump"
        );

        let _ = drive(&mut c, &last_event, &mut simulate).unwrap();
        assert_eq!(
            c.field.get(jumper).points,
            None,
            "points still unchanged after second HumanJump"
        );
    }

    #[test]
    fn drive_after_manual_record_advances_to_next_state() {
        let mut c = make_competition(65, 10, vec![0]);
        c.advance();

        let last_event = Cell::new(0);
        let mut simulate = |_: JumpParticipant, _: usize| -> Result<JumpOutcome, &'static str> {
            panic!("should not be called");
        };

        // Emulate what record_finished_human_jump + advance does
        // after acknowledgement
        c.record_jump(CompetitionJumpOutcome {
            score: 100.0,
            distance: 90.0,
            fall_type: FallType::None,
        });
        c.advance();

        // Drive should now skip simulated computers until the next
        // visible state (HumanJump for next human, or ShowResults)
        let result = drive(&mut c, &last_event, &mut simulate).unwrap();
        match result {
            WorldCupCommand::HumanJump { .. }
            | WorldCupCommand::ShowResults
            | WorldCupCommand::Done => {}
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
