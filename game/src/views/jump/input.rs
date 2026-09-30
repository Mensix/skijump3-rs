use crate::jump::types::BarAnimation;
use crate::jump::{JumpInput, JumpPhase, JumpRunner};
use crate::save::config::Config;
use crate::ui::{Key, UiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpInputAction {
    None,
    ResetWind,
    ResetJump,
    PersistStartGate(i32),
    SaveReplay,
    Consumed,
    Aborted,
    OpenSetup,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JumpInputController;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JumpKeyBindings {
    up: i32,
    right: i32,
    left: i32,
    telemark: i32,
    replay: i32,
}

impl JumpKeyBindings {
    pub(crate) fn from_config(config: &Config) -> Self {
        Self {
            up: config.key_up,
            right: config.key_right,
            left: config.key_left,
            telemark: config.key_telemark,
            replay: config.key_replay,
        }
    }

    pub(crate) fn code_for(event: UiEvent) -> Option<i32> {
        match event {
            UiEvent::KeyDown(Key::Up) => Some(72),
            UiEvent::KeyDown(Key::Right) => Some(77),
            UiEvent::KeyDown(Key::Left) => Some(75),
            UiEvent::KeyDown(Key::Down) => Some(80),
            UiEvent::KeyDown(Key::Home) => Some(71),
            UiEvent::KeyDown(Key::End) => Some(79),
            UiEvent::KeyDown(Key::Insert) => Some(82),
            UiEvent::KeyDown(Key::PageUp) => Some(73),
            UiEvent::KeyDown(Key::PageDown) => Some(81),
            UiEvent::KeyDown(Key::Kp5) => Some(76),
            UiEvent::KeyDown(Key::Backspace) => Some((8 << 8) + 14),
            UiEvent::KeyDown(Key::Delete) => Some(83),
            UiEvent::KeyDown(Key::Tab) => Some((9 << 8) + 15),
            UiEvent::KeyDown(Key::F1) => Some(59),
            UiEvent::KeyDown(Key::F2) => Some(60),
            UiEvent::KeyDown(Key::F3) => Some(61),
            UiEvent::KeyDown(Key::F4) => Some(62),
            UiEvent::KeyDown(Key::F5) => Some(63),
            UiEvent::KeyDown(Key::F6) => Some(64),
            UiEvent::KeyDown(Key::F7) => Some(65),
            UiEvent::KeyDown(Key::F8) => Some(66),
            UiEvent::KeyDown(Key::F9) => Some(67),
            UiEvent::Text(c) => char_key_code(c),
            _ => None,
        }
    }

    fn matches_up(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.up)
    }

    fn matches_right(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.right)
    }

    fn matches_left(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.left)
    }

    fn matches_telemark(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.telemark)
    }

    fn matches_replay(self, event: UiEvent) -> bool {
        Self::code_for(event) == Some(self.replay)
    }
}

pub(crate) fn char_key_code(c: char) -> Option<i32> {
    let c = c.to_ascii_uppercase();
    let scan = match c {
        'A' => 30,
        'B' => 48,
        'C' => 46,
        'D' => 32,
        'E' => 18,
        'F' => 33,
        'G' => 34,
        'H' => 35,
        'I' => 23,
        'J' => 36,
        'K' => 37,
        'L' => 38,
        'M' => 50,
        'N' => 49,
        'O' => 24,
        'P' => 25,
        'Q' => 16,
        'R' => 19,
        'S' => 31,
        'T' => 20,
        'U' => 22,
        'V' => 47,
        'W' => 17,
        'X' => 45,
        'Y' => 21,
        'Z' => 44,
        '0' => 11,
        '1' => 2,
        '2' => 3,
        '3' => 4,
        '4' => 5,
        '5' => 6,
        '6' => 7,
        '7' => 8,
        '8' => 9,
        '9' => 10,
        ' ' => 57,
        '.' => 52,
        ',' => 51,
        '-' => 12,
        '+' => 13,
        '/' => 53,
        '*' => 55,
        _ => return None,
    };
    Some((c as i32) * 256 + scan)
}

impl JumpInputController {
    pub(crate) fn handle_event(
        self,
        event: UiEvent,
        runner: &mut JumpRunner,
        keys: JumpKeyBindings,
        allow_special_wind_reset: bool,
    ) -> JumpInputAction {
        match event {
            UiEvent::KeyDown(Key::Escape) => Self::escape(runner),
            _ if pauses_jump(event, runner.phase()) => JumpInputAction::Consumed,
            UiEvent::KeyDown(Key::F5) => {
                if runner.policy().allow_wind_reset || allow_special_wind_reset {
                    JumpInputAction::ResetWind
                } else if runner.phase() == Some(JumpPhase::Info) {
                    Self::leave_info(runner)
                } else {
                    JumpInputAction::None
                }
            }
            UiEvent::KeyDown(Key::Enter) => Self::enter(runner),
            UiEvent::Text('+') => Self::adjust_gate(runner, 1),
            UiEvent::Text('-') => Self::adjust_gate(runner, -1),
            _ if keys.matches_right(event) => Self::right(runner),
            _ if keys.matches_left(event) => {
                match runner.phase() {
                    Some(JumpPhase::Info) => return Self::leave_info(runner),
                    Some(JumpPhase::OnBar) => {
                        if let Some(animation) = bar_animation_for_event(event, keys) {
                            runner.handle_input(JumpInput::BarAnimation(animation));
                        }
                    }
                    Some(JumpPhase::Flight) => runner.handle_input(JumpInput::LeanBack),
                    _ => {}
                }
                JumpInputAction::None
            }
            _ if keys.matches_up(event) => {
                match runner.phase() {
                    Some(JumpPhase::Info) => return Self::leave_info(runner),
                    Some(JumpPhase::OnBar) => {
                        if let Some(animation) = bar_animation_for_event(event, keys) {
                            runner.handle_input(JumpInput::BarAnimation(animation));
                        }
                    }
                    Some(JumpPhase::Inrun) => runner.handle_input(JumpInput::Takeoff),
                    _ => {}
                }
                JumpInputAction::None
            }
            _ if keys.matches_telemark(event) => {
                match runner.phase() {
                    Some(JumpPhase::Info) => return Self::leave_info(runner),
                    Some(JumpPhase::OnBar) => {
                        if let Some(animation) = bar_animation_for_event(event, keys) {
                            runner.handle_input(JumpInput::BarAnimation(animation));
                        }
                    }
                    Some(JumpPhase::Flight) => runner.handle_input(JumpInput::Telemark),
                    _ => {}
                }
                JumpInputAction::None
            }
            _ if keys.matches_replay(event) => {
                match runner.phase() {
                    Some(JumpPhase::Info) => return Self::leave_info(runner),
                    Some(JumpPhase::Flight) => runner.handle_input(JumpInput::TwoFooted),
                    _ => {}
                }
                JumpInputAction::None
            }
            UiEvent::KeyDown(_) | UiEvent::Text(_) if runner.phase() == Some(JumpPhase::Info) => {
                Self::leave_info(runner)
            }
            _ => JumpInputAction::None,
        }
    }

    fn enter(runner: &mut JumpRunner) -> JumpInputAction {
        match runner.phase() {
            Some(JumpPhase::Result) => JumpInputAction::ResetJump,
            Some(JumpPhase::Info) => Self::leave_info(runner),
            Some(JumpPhase::Landing) => {
                runner.handle_input(JumpInput::ShowResult);
                JumpInputAction::None
            }
            _ => {
                runner.handle_input(JumpInput::Start);
                JumpInputAction::None
            }
        }
    }

    fn leave_info(runner: &mut JumpRunner) -> JumpInputAction {
        let start_gate = runner.start_gate().unwrap_or_default();
        runner.handle_input(JumpInput::LeaveInfo);
        JumpInputAction::PersistStartGate(start_gate)
    }

    fn escape(runner: &mut JumpRunner) -> JumpInputAction {
        let phase = runner.phase();
        if aborts_jump(phase, runner.participant_is_computer()) {
            runner.handle_input(JumpInput::Abort);
            return JumpInputAction::Aborted;
        }
        match phase {
            Some(JumpPhase::Info) => Self::leave_info(runner),
            Some(JumpPhase::Landing) => {
                runner.handle_input(JumpInput::ShowResult);
                JumpInputAction::Consumed
            }
            _ => JumpInputAction::None,
        }
    }

    fn right(runner: &mut JumpRunner) -> JumpInputAction {
        match runner.phase() {
            Some(JumpPhase::Info) => {
                let start_gate = runner.start_gate().unwrap_or_default();
                runner.handle_input(JumpInput::LeaveInfo);
                JumpInputAction::PersistStartGate(start_gate)
            }
            Some(JumpPhase::OnBar) => {
                runner.handle_input(JumpInput::Start);
                JumpInputAction::None
            }
            _ => {
                runner.handle_input(JumpInput::LeanForward);
                JumpInputAction::None
            }
        }
    }

    fn adjust_gate(runner: &mut JumpRunner, delta: i32) -> JumpInputAction {
        if runner.phase() != Some(JumpPhase::Info) {
            return JumpInputAction::None;
        }
        if !runner.policy().allow_start_gate_adjust {
            return Self::leave_info(runner);
        }
        runner
            .handle_start_gate_adjust(delta)
            .map_or(JumpInputAction::None, JumpInputAction::PersistStartGate)
    }
}

fn bar_animation_for_event(event: UiEvent, keys: JumpKeyBindings) -> Option<BarAnimation> {
    if keys.matches_up(event) {
        Some(BarAnimation::Up)
    } else if keys.matches_left(event) {
        Some(BarAnimation::Left)
    } else if keys.matches_telemark(event) {
        Some(BarAnimation::Telemark)
    } else {
        None
    }
}

fn pauses_jump(event: UiEvent, phase: Option<JumpPhase>) -> bool {
    matches!(event, UiEvent::Text('p' | 'P'))
        && matches!(phase, Some(JumpPhase::Flight | JumpPhase::Landing))
}

fn aborts_jump(phase: Option<JumpPhase>, is_computer: bool) -> bool {
    !is_computer
        && matches!(
            phase,
            Some(JumpPhase::OnBar | JumpPhase::Inrun | JumpPhase::Flight)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::input::key_name;
    use crate::text::lang::LangBase;

    fn test_lang() -> LangBase {
        let mut strings = vec!["?".to_string(); 284];
        strings[280] = "ARROW UP".to_string();
        strings[281] = "ARROW LEFT".to_string();
        strings[282] = "ARROW RIGHT".to_string();
        strings[283] = "ARROW DOWN".to_string();
        LangBase::new(vec![strings], Vec::new(), 0)
    }

    fn supported_keys() -> Vec<(UiEvent, i32, String)> {
        let mut keys = vec![
            (UiEvent::KeyDown(Key::F1), 59, "F1".to_string()),
            (UiEvent::KeyDown(Key::F2), 60, "F2".to_string()),
            (UiEvent::KeyDown(Key::F3), 61, "F3".to_string()),
            (UiEvent::KeyDown(Key::F4), 62, "F4".to_string()),
            (UiEvent::KeyDown(Key::F5), 63, "F5".to_string()),
            (UiEvent::KeyDown(Key::F6), 64, "F6".to_string()),
            (UiEvent::KeyDown(Key::F7), 65, "F7".to_string()),
            (UiEvent::KeyDown(Key::F8), 66, "F8".to_string()),
            (UiEvent::KeyDown(Key::F9), 67, "F9".to_string()),
            (UiEvent::KeyDown(Key::Home), 71, "HOME".to_string()),
            (UiEvent::KeyDown(Key::End), 79, "END".to_string()),
            (UiEvent::KeyDown(Key::Insert), 82, "INSERT".to_string()),
            (UiEvent::KeyDown(Key::Delete), 83, "DELETE".to_string()),
            (UiEvent::KeyDown(Key::PageUp), 73, "PAGE UP".to_string()),
            (UiEvent::KeyDown(Key::PageDown), 81, "PAGE DOWN".to_string()),
            (UiEvent::KeyDown(Key::Kp5), 76, "NP 5".to_string()),
            (UiEvent::KeyDown(Key::Up), 72, "ARROW UP".to_string()),
            (UiEvent::KeyDown(Key::Right), 77, "ARROW RIGHT".to_string()),
            (UiEvent::KeyDown(Key::Left), 75, "ARROW LEFT".to_string()),
            (UiEvent::KeyDown(Key::Down), 80, "ARROW DOWN".to_string()),
            (
                UiEvent::KeyDown(Key::Backspace),
                (8 << 8) + 14,
                "BACKSPACE".to_string(),
            ),
            (UiEvent::KeyDown(Key::Tab), (9 << 8) + 15, "TAB".to_string()),
        ];

        for (c, scan) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().zip([
            30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47,
            17, 45, 21, 44,
        ]) {
            let code = (c as i32) * 256 + scan;
            keys.push((UiEvent::Text(c), code, c.to_string()));
            keys.push((UiEvent::Text(c.to_ascii_lowercase()), code, c.to_string()));
        }
        for (c, scan) in "0123456789".chars().zip([11, 2, 3, 4, 5, 6, 7, 8, 9, 10]) {
            keys.push((UiEvent::Text(c), (c as i32) * 256 + scan, c.to_string()));
        }
        for (c, scan, name) in [
            (' ', 57, "SPACE"),
            ('.', 52, "."),
            (',', 51, ","),
            ('-', 12, "-"),
            ('+', 13, "+"),
            ('/', 53, "/"),
            ('*', 55, "*"),
        ] {
            keys.push((UiEvent::Text(c), (c as i32) * 256 + scan, name.to_string()));
        }
        keys
    }

    #[test]
    fn every_pascal_configurable_key_roundtrips_and_matches() {
        let lang = test_lang();
        for (event, expected_code, expected_name) in supported_keys() {
            let code = JumpKeyBindings::code_for(event);
            assert_eq!(code, Some(expected_code), "event {event:?}");
            assert_eq!(key_name(expected_code, &lang), expected_name);

            let bindings = JumpKeyBindings {
                up: expected_code,
                right: expected_code,
                left: expected_code,
                telemark: expected_code,
                replay: expected_code,
            };
            assert!(bindings.matches_up(event));
            assert!(bindings.matches_right(event));
            assert!(bindings.matches_left(event));
            assert!(bindings.matches_telemark(event));
            assert!(bindings.matches_replay(event));
        }
    }

    #[test]
    fn f10_remains_reserved_and_cannot_match_a_jump_binding() {
        let event = UiEvent::KeyDown(Key::F10);
        let bindings = JumpKeyBindings {
            up: 68,
            right: 68,
            left: 68,
            telemark: 68,
            replay: 68,
        };

        assert_eq!(JumpKeyBindings::code_for(event), None);
        assert!(!bindings.matches_up(event));
        assert!(!bindings.matches_right(event));
        assert!(!bindings.matches_left(event));
        assert!(!bindings.matches_telemark(event));
        assert!(!bindings.matches_replay(event));
    }

    #[test]
    fn p_pauses_only_flight_and_landing() {
        assert!(pauses_jump(UiEvent::Text('p'), Some(JumpPhase::Flight)));
        assert!(pauses_jump(UiEvent::Text('P'), Some(JumpPhase::Landing)));
        assert!(!pauses_jump(UiEvent::Text('p'), Some(JumpPhase::Inrun)));
        assert!(!pauses_jump(
            UiEvent::KeyDown(Key::Escape),
            Some(JumpPhase::Flight)
        ));
    }

    #[test]
    fn escape_aborts_only_active_human_jump() {
        assert!(aborts_jump(Some(JumpPhase::OnBar), false));
        assert!(aborts_jump(Some(JumpPhase::Inrun), false));
        assert!(aborts_jump(Some(JumpPhase::Flight), false));
        assert!(!aborts_jump(Some(JumpPhase::Landing), false));
        assert!(!aborts_jump(Some(JumpPhase::Flight), true));
    }

    #[test]
    fn original_three_controls_select_bar_animation_families() {
        let keys = JumpKeyBindings {
            up: 72,
            right: 77,
            left: 75,
            telemark: char_key_code('T').unwrap(),
            replay: char_key_code('R').unwrap(),
        };

        assert_eq!(
            bar_animation_for_event(UiEvent::KeyDown(Key::Up), keys),
            Some(BarAnimation::Up)
        );
        assert_eq!(
            bar_animation_for_event(UiEvent::KeyDown(Key::Left), keys),
            Some(BarAnimation::Left)
        );
        assert_eq!(
            bar_animation_for_event(UiEvent::Text('t'), keys),
            Some(BarAnimation::Telemark)
        );
        assert_eq!(
            bar_animation_for_event(UiEvent::KeyDown(Key::Right), keys),
            None
        );
    }
}
