use crate::jump::animation::landing_height;
use crate::jump::math::{self, nsqrt};
use crate::jump::types::{JumpInput, JumpPhase, JumpSnapshot};
use crate::rng::Random;

pub trait JumpInputProvider {
    fn inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut Random) -> Vec<JumpInput>;
}

#[derive(Debug)]
pub struct ComputerInputProvider {
    pascal_jumper: i32,
    started: bool,
    initialized: bool,
    skill: i32,
    reflex: i32,
    planned_two_footed: bool,
    takeoff_started: bool,
}

impl ComputerInputProvider {
    pub(crate) const fn new(participant_id: usize) -> Self {
        Self {
            pascal_jumper: participant_id as i32 + 1,
            started: false,
            initialized: false,
            skill: 16,
            reflex: 7,
            planned_two_footed: false,
            takeoff_started: false,
        }
    }

    pub(crate) fn prepare_for_jump(&mut self, rng: &mut Random) {
        self.initialize(rng);
    }

    fn initialize(&mut self, rng: &mut Random) {
        if self.initialized {
            return;
        }

        let jumper = self.pascal_jumper.max(1);
        let mut skill_raw = rng.random_i32(80);
        skill_raw -= rng.random_i32(24 * jumper);
        skill_raw -= rng.random_i32(12 * jumper);
        skill_raw -= rng.random_i32(10 * jumper);
        skill_raw = 63 - skill_raw;

        self.skill = 17 - math::round(nsqrt(f64::from(skill_raw)) / 4.0);
        if self.skill > 16 {
            self.skill = 16;
        }

        self.reflex = math::round(f64::from(34 + jumper + rng.random_i32(10)) / 5.0).max(1);

        if rng.random_i32(300 - jumper) == 0 {
            self.skill = 17 + rng.random_i32(3 + jumper / 25);
        }

        self.initialized = true;
    }

    fn flight_inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut Random) -> Vec<JumpInput> {
        let mut inputs = Vec::with_capacity(2);

        if snapshot.frame % self.reflex == 0 {
            if snapshot.body_angle >= 62 {
                inputs.push(JumpInput::LeanForward);
            } else if snapshot.body_angle < 50 {
                inputs.push(JumpInput::LeanBack);
            }
        }

        let mut landing_height = landing_height(snapshot.slope_angle);
        if self.planned_two_footed {
            landing_height = math::round(f64::from(landing_height) * 0.6);
        }

        if snapshot.table_distance > 3.0 && snapshot.height < 4 {
            inputs.push(JumpInput::TwoFooted);
        } else if snapshot.delta_height_sum > 1 && snapshot.height < landing_height {
            if self.planned_two_footed {
                inputs.push(JumpInput::TwoFooted);
            } else if rng.random_i32(250) < landing_height * (landing_height / 10) {
                self.planned_two_footed = true;
            } else {
                inputs.push(JumpInput::Telemark);
            }
        }

        inputs
    }
}

impl JumpInputProvider for ComputerInputProvider {
    fn inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut Random) -> Vec<JumpInput> {
        self.initialize(rng);

        match snapshot.phase {
            JumpPhase::Info => vec![JumpInput::LeaveInfo],
            JumpPhase::OnBar if !self.started => {
                self.started = true;
                vec![JumpInput::Start]
            }
            JumpPhase::Inrun
                if !self.takeoff_started
                    && snapshot.table_distance
                        > -(f64::from(self.skill) * snapshot.speed * 0.01) =>
            {
                self.takeoff_started = true;
                vec![JumpInput::Takeoff]
            }
            JumpPhase::Flight => self.flight_inputs(snapshot, rng),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::hill_profile::HillTerrain;
    use crate::jump::types::FlightWind;
    use crate::jump::JumpState;
    use crate::parsers::pcx::PcxParser;
    use crate::parsers::AssetParser;

    fn snapshot(phase: JumpPhase) -> JumpSnapshot {
        JumpSnapshot {
            phase,
            frame: 1,
            x: 0,
            table_distance: -100.0,
            y: 0,
            height: 100,
            delta_height_sum: 0,
            slope_angle: 30,
            distance: 0,
            body_angle: 158,
            ski_angle: 0,
            speed: 90.0,
            start_gate: 15,
        }
    }

    #[test]
    fn computer_starts_takeoff_near_table() {
        let mut provider = ComputerInputProvider::new(1);
        let mut rng = Random::new(1);
        let mut near_table = snapshot(JumpPhase::Inrun);
        near_table.table_distance = -5.0;

        assert_eq!(
            provider.inputs(&near_table, &mut rng),
            vec![JumpInput::Takeoff]
        );
    }

    #[test]
    fn computer_does_not_takeoff_too_early() {
        let mut provider = ComputerInputProvider::new(1);
        let mut rng = Random::new(1);
        let far_from_table = snapshot(JumpPhase::Inrun);

        assert!(provider.inputs(&far_from_table, &mut rng).is_empty());
    }

    #[test]
    fn computer_corrects_flight_angle_on_reflex_tick() {
        let mut provider = ComputerInputProvider::new(1);
        provider.initialized = true;
        provider.reflex = 4;
        let mut rng = Random::new(1);
        let mut flight = snapshot(JumpPhase::Flight);
        flight.frame = 8;
        flight.body_angle = 158;

        assert!(provider
            .inputs(&flight, &mut rng)
            .contains(&JumpInput::LeanForward));
    }

    #[test]
    fn computer_simulation_gets_past_table_fall_distance() {
        let front = PcxParser::parse(include_bytes!("../../assets/FRONT1.PCX")).expect("FRONT1");
        let terrain = HillTerrain::from_front_pcx(front, 120, 0.89);
        let mut state = JumpState::new(&terrain, 148.0, 0.89, 120, 0.3217, 15);
        let mut provider = ComputerInputProvider::new(1);
        let mut rng = Random::new(1);
        let wind = FlightWind {
            value: 0,
            windy: 0,
            strength: 0,
        };

        for _ in 0..5000 {
            let snapshot = state.snapshot_with_terrain(&terrain);
            for input in provider.inputs(&snapshot, &mut rng) {
                state.handle_input(input);
            }
            state.tick(&terrain, wind, &mut rng, true);
            if let Some(outcome) = state.outcome() {
                assert!(
                    outcome.distance > 700,
                    "computer jump landed too short: {}",
                    outcome.distance
                );
                return;
            }
        }

        panic!("computer simulation did not finish");
    }

    #[test]
    fn silent_computer_jump_starts_like_pascal_non_view_path() {
        let front = PcxParser::parse(include_bytes!("../../assets/FRONT2.PCX")).expect("FRONT2");
        let terrain = HillTerrain::from_front_pcx(front, 90, 0.84);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);

        state.prepare_silent_computer_jump(&terrain);

        assert_eq!(state.phase, JumpPhase::Inrun);
        assert_eq!(state.frame, 0);
        assert_eq!(state.travel, -45.0);
        assert_eq!(state.px, 131.0);
    }

    #[test]
    fn silent_computer_jump_keeps_pascal_maxspeed_on_first_tick() {
        let front = PcxParser::parse(include_bytes!("../../assets/FRONT2.PCX")).expect("FRONT2");
        let terrain = HillTerrain::from_front_pcx(front, 90, 0.84);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);
        let mut rng = Random::new(1);
        let wind = FlightWind {
            value: 0,
            windy: 0,
            strength: 0,
        };

        state.prepare_silent_computer_jump(&terrain);
        state.tick(&terrain, wind, &mut rng, true);

        assert_eq!(
            state.px, 131.0,
            "silent path should not reset px to visible-start speed"
        );
    }

    #[test]
    fn silent_lahti_k90_computer_distance_stays_plausible() {
        use crate::jump::wind::Wind;

        let front = PcxParser::parse(include_bytes!("../../assets/FRONT2.PCX")).expect("FRONT2");
        let terrain = HillTerrain::from_front_pcx(front, 90, 0.84);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);
        state.prepare_silent_computer_jump(&terrain);
        let mut provider = ComputerInputProvider::new(0);
        let mut rng = Random::new(5489);
        provider.prepare_for_jump(&mut rng);
        let mut wind = Wind::default();
        wind.initialize(&mut rng, 0);
        wind.advance_without_sampling(&mut rng);
        for _ in 0..100 {
            wind.advance_without_sampling(&mut rng);
        }

        for _ in 0..5000 {
            let snapshot = state.snapshot_with_terrain(&terrain);
            for input in provider.inputs(&snapshot, &mut rng) {
                state.handle_input(input);
            }
            let sampled = FlightWind {
                value: wind.sample(&mut rng),
                windy: wind.windy,
                strength: wind.strength,
            };
            state.tick(&terrain, sampled, &mut rng, true);
            if let Some(outcome) = state.outcome() {
                assert_eq!(outcome.distance, 945);
                return;
            }
        }

        panic!("silent Lahti K90 computer simulation did not finish");
    }

    #[test]
    fn silent_computer_skips_landing_phase() {
        let front = PcxParser::parse(include_bytes!("../../assets/FRONT2.PCX")).expect("FRONT2");
        let terrain = HillTerrain::from_front_pcx(front, 90, 0.84);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);
        let mut rng = Random::new(1);
        let wind = FlightWind {
            value: 0,
            windy: 0,
            strength: 0,
        };

        state.prepare_silent_computer_jump(&terrain);
        for _ in 0..5000 {
            let snapshot = state.snapshot_with_terrain(&terrain);
            if snapshot.phase == JumpPhase::Info {
                state.handle_input(JumpInput::LeaveInfo);
            } else if snapshot.phase == JumpPhase::OnBar {
                state.handle_input(JumpInput::Start);
            } else if snapshot.phase == JumpPhase::Inrun && snapshot.table_distance > -20.0 {
                state.handle_input(JumpInput::Takeoff);
            } else if snapshot.phase == JumpPhase::Flight {
                state.handle_input(JumpInput::Telemark);
            }
            state.tick(&terrain, wind, &mut rng, true);
            if let Some(_) = state.outcome() {
                assert_eq!(state.phase, JumpPhase::Result);
                assert_eq!(state.landing_counter, 0);
                return;
            }
        }
        panic!("silent computer never reached Result");
    }

    #[test]
    fn pre_start_wind_shift_matches_pascal_count() {
        use crate::jump::wind::Wind;

        let mut rng = Random::new(42);
        let mut wind = Wind::default();
        wind.initialize(&mut rng, 0);

        // Pascal: 1x Tuuli.Hae (shift) + 100x Tuuli.Siirra (shift)
        // before the first physics-relevant wind sample
        wind.advance_without_sampling(&mut rng);
        for _ in 0..100 {
            wind.advance_without_sampling(&mut rng);
        }

        // The angle has moved 101 steps; a fresh sample produces a valid value
        let value = wind.sample(&mut rng);
        assert!(value >= -50 && value <= 50, "wind value out of range: {value}");
    }
}
