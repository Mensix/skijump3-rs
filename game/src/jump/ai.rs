use crate::jump::animation::landing_height;
use crate::jump::math::{self, nsqrt};
use crate::jump::types::{BarAnimation, JumpInput, JumpPhase, JumpSnapshot, LandingStyle};
use crate::rng::Random;

#[cfg(test)]
use crate::jump::types::DEFAULT_START_GATE;

#[derive(Debug)]
pub struct ComputerInputProvider {
    jumper_id: i32,
    start_requested: bool,
    initialized: bool,
    skill: i32,
    reflex: i32,
    planned_two_footed: bool,
    takeoff_started: bool,
}

impl ComputerInputProvider {
    pub(crate) const fn new(participant_id: usize) -> Self {
        Self {
            jumper_id: participant_id as i32,
            start_requested: false,
            initialized: false,
            skill: 16,
            reflex: 7,
            planned_two_footed: false,
            takeoff_started: false,
        }
    }

    pub(crate) fn initialize(&mut self, rng: &mut Random) {
        if self.initialized {
            return;
        }

        let jumper = self.jumper_id.max(0) + 1;
        let skill_raw = 63 - rng.random_i32(80)
            + rng.random_i32(24 * jumper)
            + rng.random_i32(12 * jumper)
            + rng.random_i32(10 * jumper);

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

        let mut landing_target = landing_height(snapshot.slope_angle);
        if self.planned_two_footed {
            landing_target = math::round(f64::from(landing_target) * 0.6);
        }

        if snapshot.table_distance > 3.0
            && snapshot.height < 4
            && !self.planned_two_footed
            && snapshot.landing_style == LandingStyle::None
        {
            inputs.push(JumpInput::TwoFooted);
        } else if snapshot.delta_height_sum > 1 && snapshot.height < landing_target {
            if self.planned_two_footed {
                inputs.push(JumpInput::TwoFooted);
            } else if rng.random_i32(250) < landing_target * (landing_target / 10) {
                self.planned_two_footed = true;
            } else {
                inputs.push(JumpInput::Telemark);
            }
        }

        inputs
    }

    pub(crate) fn inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut Random) -> Vec<JumpInput> {
        self.initialize(rng);

        match snapshot.phase {
            JumpPhase::Info => Vec::new(),
            JumpPhase::OnBar => self.on_bar_inputs(snapshot, rng),
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

    fn on_bar_inputs(&mut self, snapshot: &JumpSnapshot, rng: &mut Random) -> Vec<JumpInput> {
        let mut inputs = Vec::with_capacity(2);
        let start_roll = rng.random_i32(100);
        if !self.start_requested && (start_roll == 0 || snapshot.frame > 600) {
            self.start_requested = true;
            inputs.push(JumpInput::Start);
        }

        if rng.random_i32(100) == 0 && !snapshot.bar_animation_active {
            let animation = match rng.random_i32(3) {
                0 => BarAnimation::Up,
                1 => BarAnimation::Telemark,
                _ => BarAnimation::Left,
            };
            inputs.push(JumpInput::BarAnimation(animation));
        }
        inputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::hill_profile::HillTerrain;
    use crate::files::FileStore;
    use crate::jump::state::JumpState;
    use crate::jump::types::FlightWind;

    fn test_files() -> FileStore {
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        FileStore::new(assets, std::path::PathBuf::from("."))
    }

    fn hill_loader(idx: usize) -> HillTerrain {
        let idx = idx.to_string();
        HillTerrain::load_with_markers(&test_files(), &idx, &idx, 100, false, 0, 0.0)
    }

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
            distance: 0.0,
            body_angle: 158,
            ski_angle: 0,
            speed: 90.0,
            start_gate: DEFAULT_START_GATE,
            bar_animation_active: false,
            landing_style: LandingStyle::None,
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
    fn visible_computer_waits_on_info_for_acknowledgement() {
        let mut provider = ComputerInputProvider::new(1);
        let mut rng = Random::new(1);

        assert!(provider
            .inputs(&snapshot(JumpPhase::Info), &mut rng)
            .is_empty());
    }

    #[test]
    fn computer_on_bar_consumes_both_pascal_random_rolls() {
        let mut provider = ComputerInputProvider::new(1);
        provider.initialized = true;
        let mut on_bar = snapshot(JumpPhase::OnBar);
        on_bar.frame = 601;
        let mut actual = Random::new(1);
        let mut expected = actual.clone();
        expected.random_i32(100);
        expected.random_i32(100);

        assert_eq!(
            provider.inputs(&on_bar, &mut actual),
            vec![JumpInput::Start]
        );
        assert_eq!(actual.random_i32(1_000_000), expected.random_i32(1_000_000));
    }

    #[test]
    fn computer_bar_animation_consumes_family_roll() {
        let seed = (0..10_000)
            .find(|&seed| {
                let mut rng = Random::new(seed);
                rng.random_i32(100) != 0 && rng.random_i32(100) == 0
            })
            .expect("a deterministic animation seed");
        let mut provider = ComputerInputProvider::new(1);
        provider.initialized = true;
        let mut actual = Random::new(seed);
        let mut expected = actual.clone();
        expected.random_i32(100);
        expected.random_i32(100);
        let family = expected.random_i32(3);

        assert_eq!(
            provider.inputs(&snapshot(JumpPhase::OnBar), &mut actual),
            vec![JumpInput::BarAnimation(match family {
                0 => BarAnimation::Up,
                1 => BarAnimation::Telemark,
                _ => BarAnimation::Left,
            })]
        );
        assert_eq!(actual.random_i32(1_000_000), expected.random_i32(1_000_000));
    }

    #[test]
    fn lower_roster_ids_get_better_takeoff_skill() {
        let mut best = ComputerInputProvider::new(0);
        let mut worst = ComputerInputProvider::new(64);

        best.initialize(&mut Random::new(5489));
        worst.initialize(&mut Random::new(5489));

        assert!(
            best.skill > worst.skill,
            "best skill {} should beat worst skill {}",
            best.skill,
            worst.skill
        );
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
        let terrain = hill_loader(0);
        let mut state = JumpState::new(&terrain, 148.0, 0.89, 120, 0.3217, 15);
        let mut provider = ComputerInputProvider::new(1);
        let mut rng = Random::new(1);
        let wind = FlightWind {
            value: 0,
            windy: 0,
            strength: 0,
        };
        state.handle_input(JumpInput::LeaveInfo);

        for _ in 0..5000 {
            let snapshot = state.snapshot_with_terrain(&terrain);
            for input in provider.inputs(&snapshot, &mut rng) {
                state.handle_input(input);
            }
            state.tick(&terrain, wind, &mut rng, true);
            if let Some(outcome) = state.outcome() {
                assert!(
                    outcome.distance > 70.0,
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
        let terrain = hill_loader(1);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);

        state.prepare_silent_computer_jump(&terrain);

        assert_eq!(state.phase, JumpPhase::Inrun);
        assert_eq!(state.frame, 0);
        assert!((state.travel - (-45.0)).abs() < f64::EPSILON);
        assert!((state.px - 131.0).abs() < f64::EPSILON);
    }

    #[test]
    fn silent_computer_jump_keeps_pascal_maxspeed_on_first_tick() {
        let terrain = hill_loader(1);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);
        let mut rng = Random::new(1);
        let wind = FlightWind {
            value: 0,
            windy: 0,
            strength: 0,
        };

        state.prepare_silent_computer_jump(&terrain);
        state.tick(&terrain, wind, &mut rng, true);

        assert!(
            (state.px - 131.0).abs() < f64::EPSILON,
            "silent path should not reset px to visible-start speed"
        );
    }

    #[test]
    fn silent_lahti_k90_computer_distance_stays_plausible() {
        use crate::jump::wind::Wind;

        let terrain = hill_loader(1);
        let mut state = JumpState::new(&terrain, 131.0, 0.84, 90, 0.3222, 15);
        state.prepare_silent_computer_jump(&terrain);
        let mut provider = ComputerInputProvider::new(0);
        let mut rng = Random::new(5489);
        provider.initialize(&mut rng);
        let mut wind = Wind::default();
        wind.initialize(&mut rng, 0);
        wind.advance(&mut rng);
        for _ in 0..100 {
            wind.advance(&mut rng);
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
                assert_eq!(outcome.distance, 93.5);
                return;
            }
        }

        panic!("silent Lahti K90 computer simulation did not finish");
    }

    #[test]
    fn silent_computer_skips_landing_phase() {
        let terrain = hill_loader(1);
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
            if state.outcome().is_some() {
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

        wind.advance(&mut rng);
        for _ in 0..100 {
            wind.advance(&mut rng);
        }

        let value = wind.sample(&mut rng);
        assert!(
            (-50..=50).contains(&value),
            "wind value out of range: {value}"
        );
    }
}
