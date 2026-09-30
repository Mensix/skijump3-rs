use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::jump::ai::ComputerInputProvider;
use crate::jump::config::JumpParticipant;
use crate::jump::state::JumpState;
use crate::jump::types::{FallType, JumpOutcome, JumpPhase, LandingStyle};
use crate::jump::wind::Wind;
use crate::rng::Random;

const MAX_SIMULATION_TICKS: usize = 10_000;

pub(crate) const fn aborted_outcome() -> JumpOutcome {
    JumpOutcome {
        distance: 0.0,
        score: 0.0,
        style_points: [0.0; 5],
        landing_style: LandingStyle::None,
        fall_type: FallType::None,
        injury: 0,
        aborted: true,
    }
}

pub(crate) fn simulate_computer_with_pre_jump_wind(
    participant: &JumpParticipant,
    terrain: &HillTerrain,
    hill: &HillInfo,
    rng: &mut Random,
    wind: &mut Wind,
    advance_pre_jump_wind: bool,
) -> JumpOutcome {
    let mut state = JumpState::new(
        terrain,
        hill.vx_final as f64,
        hill.pk(),
        hill.kr as i32,
        hill.pl_save(),
        15,
    );
    let mut input = ComputerInputProvider::new(participant.ai_id);

    if advance_pre_jump_wind {
        wind.advance(rng);
    }
    input.initialize(rng);
    state.prepare_silent_computer_jump(terrain);
    for _ in 0..100 {
        wind.advance(rng);
    }

    for _ in 0..MAX_SIMULATION_TICKS {
        if let Some(outcome) = state.outcome() {
            return outcome;
        }
        let snapshot = state.snapshot_with_terrain(terrain);
        for inp in input.inputs(&snapshot, rng) {
            state.handle_input(inp);
        }
        let flight_wind = wind.sample_flight_wind(Some(state.phase), rng);
        state.tick(terrain, flight_wind, rng, true);
        if state.phase == JumpPhase::Result {
            state.tick(terrain, flight_wind, rng, true);
        }
    }
    aborted_outcome()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aborted_outcome_is_a_zeroed_abort() {
        let outcome = aborted_outcome();
        assert!(outcome.aborted);
        assert_eq!(outcome.distance, 0.0);
        assert_eq!(outcome.score, 0.0);
    }
}
