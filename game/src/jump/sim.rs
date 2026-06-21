use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::jump::ai::ComputerInputProvider;
use crate::jump::config::JumpParticipant;
use crate::jump::state::JumpState;
use crate::jump::types::{FlightWind, JumpOutcome, JumpPhase};
use crate::jump::wind::Wind;
use crate::rng::Random;

pub(crate) fn simulate_computer(
    participant: &JumpParticipant,
    terrain: &HillTerrain,
    hill: &HillInfo,
    rng: &mut Random,
    wind: &mut Wind,
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

    wind.advance_without_sampling(rng);
    input.prepare_for_jump(rng);
    state.prepare_silent_computer_jump(terrain);
    for _ in 0..100 {
        wind.advance_without_sampling(rng);
    }

    loop {
        if let Some(outcome) = state.outcome() {
            return outcome;
        }
        let snapshot = state.snapshot_with_terrain(terrain);
        for inp in input.inputs(&snapshot, rng) {
            state.handle_input(inp);
        }
        let wind_value = if matches!(
            state.phase,
            JumpPhase::Info | JumpPhase::Result | JumpPhase::Disqualified
        ) {
            wind.value
        } else {
            wind.sample(rng)
        };
        let flight_wind = FlightWind {
            value: wind_value,
            windy: wind.windy,
            strength: wind.strength,
        };
        state.tick(terrain, flight_wind, rng, true);
        if state.phase == JumpPhase::Result {
            state.tick(terrain, flight_wind, rng, true);
        }
    }
}
