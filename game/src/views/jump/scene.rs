use crate::competition::active::ActiveCompetition;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::data::hill::HillCatalog;
use crate::jump::config::JumpConfig;
use crate::jump::replay::ReplayTrace;
use crate::jump::sim;
use crate::jump::snow::{calculate_snow_count, SnowSystem};
use crate::jump::types::{JumpOutcome, JumpPhase, JumpTelemetry};
use crate::jump::wind::Wind;
use crate::jump::{JumpInput, JumpParticipant, JumpPolicy, JumpRunner};
use crate::rng::Random;
use crate::screen::Persistence;
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::UiEvent;
use crate::views::jump::input::{JumpInputAction, JumpInputController, JumpKeyBindings};
use crate::views::replay::save_dialog::{SaveAction, SaveReplayDialog};
use engine::audio::Beep;

fn coach_telemetry(grade: i32, height: i32, takeoff_timing: u8, body_angle: i32) -> JumpTelemetry {
    JumpTelemetry::new(
        grade.clamp(0, 255) as u8,
        height.clamp(0, 255) as u8,
        takeoff_timing,
        (body_angle / 10).clamp(0, 255) as u8,
    )
}

pub struct JumpScene {
    runner: JumpRunner,
    save_dialog: SaveReplayDialog,
    resources: ResourcesRef,
    telemetry: Option<JumpTelemetry>,
    paused: bool,
}

pub struct RunnerSetup {
    pub hill_idx: usize,
    pub start_gate: i32,
    pub participant: JumpParticipant,
    pub policy: JumpPolicy,
    pub phase_label: String,
}

impl JumpScene {
    fn prepare_snow(state: &mut GameState, policy: JumpPolicy) -> SnowSystem {
        let is_first = state.consume_first_jump_event();
        if policy.allow_wind_reset {
            return Self::prepare_training_weather(state);
        }
        let mut snow = SnowSystem::default();
        if is_first {
            Self::initialize_event_weather(
                &mut snow,
                &mut state.rng,
                &mut state.wind,
                state.config.wind_position as u8,
                state.config.graphics_detail == 1,
            );
        }
        snow
    }

    fn prepare_training_weather(state: &mut GameState) -> SnowSystem {
        let mut snow = SnowSystem::default();
        Self::initialize_event_weather(
            &mut snow,
            &mut state.rng,
            &mut state.wind,
            state.config.wind_position as u8,
            state.config.graphics_detail == 1,
        );
        snow
    }

    pub(crate) fn initialize_event_weather(
        snow: &mut SnowSystem,
        rng: &mut Random,
        wind: &mut Wind,
        wind_position: u8,
        low_detail: bool,
    ) {
        if wind.is_enabled() {
            wind.initialize(rng, wind_position);
        }
        let snow_count = calculate_snow_count(rng);
        snow.set_count(snow_count, rng);
        if low_detail || snow_count == 0 {
            snow.clear_count();
        }
        wind.sample(rng);
    }

    pub fn new(
        resources: ResourcesRef,
        state: &mut GameState,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
    ) -> Self {
        let snow = Self::prepare_snow(state, policy);
        let runner = Self::build_runner(
            resources.clone(),
            state,
            RunnerSetup {
                hill_idx,
                start_gate,
                participant,
                policy,
                phase_label: String::new(),
            },
            snow,
        );
        Self {
            runner,
            save_dialog: SaveReplayDialog::new(resources.clone()),
            telemetry: None,
            paused: false,
            resources,
        }
    }

    pub(crate) fn new_competition_placeholder(resources: ResourcesRef, state: &GameState) -> Self {
        let runner = Self::build_runner(
            resources.clone(),
            state,
            RunnerSetup {
                hill_idx: 0,
                start_gate: 15,
                participant: JumpParticipant::trainee(),
                policy: JumpPolicy::competition(),
                phase_label: String::new(),
            },
            SnowSystem::default(),
        );
        Self {
            runner,
            save_dialog: SaveReplayDialog::new(resources.clone()),
            telemetry: None,
            paused: false,
            resources,
        }
    }

    pub(crate) fn prepare_competition_event_weather(
        &mut self,
        rng: &mut Random,
        wind: &mut Wind,
        wind_position: u8,
        low_detail: bool,
    ) {
        let mut snow = self.runner.clone_snow();
        Self::initialize_event_weather(&mut snow, rng, wind, wind_position, low_detail);
        self.runner.replace_snow(snow);
    }

    pub(crate) fn prepare_training_attempt_weather(&mut self, state: &mut GameState) {
        let snow = Self::prepare_training_weather(state);
        self.runner.replace_snow(snow);
    }

    pub fn rebuild(
        &mut self,
        state: &mut GameState,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
        phase_label: String,
    ) {
        self.telemetry = None;
        self.paused = false;
        let snow = self.runner.clone_snow();
        self.runner = Self::build_runner(
            self.resources.clone(),
            state,
            RunnerSetup {
                hill_idx,
                start_gate,
                participant,
                policy,
                phase_label,
            },
            snow,
        );
    }

    pub fn rebuild_for_competition(
        &mut self,
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        phase_label: String,
        state: &mut GameState,
    ) {
        self.rebuild(
            state,
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::competition(),
            phase_label,
        );
    }

    pub fn set_phase_label(&mut self, label: String) {
        self.runner.set_phase_label(label);
    }

    pub fn set_team_name(&mut self, name: String) {
        self.runner.set_team_name(name);
    }

    pub fn set_suppress_info_panel(&mut self, suppress: bool) {
        self.runner.set_suppress_info_panel(suppress);
    }

    pub fn set_has_bib(&mut self, val: bool) {
        self.runner.set_has_bib(val);
    }

    pub fn set_save_hill_records(&mut self, save: bool) {
        self.runner.set_save_hill_records(save);
    }

    pub fn team_name(&self) -> &str {
        self.runner.team_name()
    }

    pub fn reset_state(&mut self, state: &GameState, start_gate: i32) {
        let hill_idx = self.runner.hill_idx();
        let hill = self.resources.hills.hill(hill_idx);
        let record_distance = hill
            .as_ref()
            .and_then(|h| state.records.hill_record(&h.record_key))
            .map_or(0.0, |r| r.len);
        let goal_distance = goal_distance(state, &self.resources.hills, hill_idx);
        self.runner
            .reset_state(start_gate, record_distance, goal_distance);
    }

    pub fn handle_jump_input(&mut self, state: &GameState, event: UiEvent) -> JumpInputAction {
        if self.paused {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                self.paused = false;
                return JumpInputAction::Consumed;
            }
            return JumpInputAction::None;
        }
        if self.phase() == Some(JumpPhase::Info)
            && is_localized_setup_key(event, self.resources.langbase.tr(60))
        {
            return JumpInputAction::OpenSetup;
        }
        if self.runner.participant_is_computer() {
            return JumpInputAction::Consumed;
        }
        if self.is_save_replay_event(event) {
            return JumpInputAction::SaveReplay;
        }
        let config = &state.config;
        let keys = JumpKeyBindings::from_config(config);
        let allow_special_wind_reset = self.phase() == Some(JumpPhase::OnBar)
            && allows_initial_custom_cup_wind_reset(state, &self.runner);
        let action = JumpInputController.handle_event(
            event,
            &mut self.runner,
            keys,
            allow_special_wind_reset,
        );
        if action == JumpInputAction::Consumed
            && matches!(event, UiEvent::Text('p' | 'P'))
            && matches!(self.phase(), Some(JumpPhase::Flight | JumpPhase::Landing))
        {
            self.paused = true;
        }
        action
    }

    pub fn is_save_replay_event(&self, event: UiEvent) -> bool {
        self.phase() == Some(JumpPhase::Result)
            && is_localized_prompt_key(event, self.resources.langbase.tr(298))
    }

    pub fn phase(&self) -> Option<JumpPhase> {
        self.runner.phase()
    }

    pub fn frame_counter(&self) -> i32 {
        self.runner.frame_counter()
    }

    pub fn outcome(&self) -> Option<JumpOutcome> {
        self.runner.outcome()
    }

    pub fn collect_telemetry(&mut self) {
        if self.telemetry.is_some() {
            return;
        }
        let Some(state) = self.runner.state() else {
            return;
        };
        self.telemetry = Some(coach_telemetry(
            state.grade,
            state.height,
            state.takeoff_counter,
            state.min_body_angle,
        ));
    }

    pub fn telemetry(&self) -> Option<JumpTelemetry> {
        self.telemetry
    }

    pub fn is_save_dialog_active(&self) -> bool {
        self.save_dialog.is_active()
    }

    pub fn open_save_dialog(&mut self) {
        let outcome = self.runner.outcome();
        let distance = outcome
            .map(|o| format!("{:.1}", o.distance))
            .unwrap_or_default();
        let hill_name = self
            .resources
            .hills
            .hill(self.runner.hill_idx())
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let author_name = replay_default_author(self.runner.participant());
        self.save_dialog.open(
            author_name,
            format!("Huge Jump in {hill_name}"),
            distance,
            hill_name,
        );
    }

    pub fn handle_save_dialog_event(&mut self, event: &UiEvent, cx: &Persistence) -> Option<bool> {
        let action = self.save_dialog.handle_event(event);
        match action {
            Some(SaveAction::SaveReplay) => {
                if let Some(trace) = self.replay_trace() {
                    self.save_dialog.write_replay(&trace, cx);
                }
                Some(true)
            }
            Some(SaveAction::Consumed) | None => Some(false),
        }
    }

    pub fn replay_trace(&self) -> Option<ReplayTrace> {
        self.runner.replay_trace()
    }

    pub fn hill_idx(&self) -> usize {
        self.runner.hill_idx()
    }

    pub fn participant_id(&self) -> usize {
        self.runner.participant_id()
    }

    pub fn participant_is_computer(&self) -> bool {
        self.runner.participant_is_computer()
    }

    pub fn participant(&self) -> &JumpParticipant {
        self.runner.participant()
    }

    pub fn simulate_hidden(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        rng: &mut Random,
        wind: &mut Wind,
    ) -> JumpOutcome {
        let terrain = self.resources.terrain(hill_idx);
        let Some(hill) = self.resources.hills.hill(hill_idx) else {
            return sim::aborted_outcome();
        };
        sim::simulate_computer_with_pre_jump_wind(&participant, &terrain, &hill, rng, wind, true)
    }

    pub fn simulate_hidden_after_visible(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        rng: &mut Random,
        wind: &mut Wind,
    ) -> JumpOutcome {
        let terrain = self.resources.terrain(hill_idx);
        let Some(hill) = self.resources.hills.hill(hill_idx) else {
            return sim::aborted_outcome();
        };
        sim::simulate_computer_with_pre_jump_wind(
            &participant,
            &terrain,
            &hill,
            rng,
            wind,
            !self.runner.pre_jump_wind_done(),
        )
    }

    pub fn render(&self, cx: &mut dyn UiCanvas, state: &GameState) {
        if self.is_save_dialog_active() {
            self.save_dialog.paint(cx);
            return;
        }
        self.runner.render(
            cx,
            &self.resources.font,
            &self.resources.langbase,
            &self.resources.hills,
            &state.records,
            &state.wind,
        )
    }

    pub(crate) fn render_darkened_hill_background(
        &self,
        cx: &mut dyn UiCanvas,
        hill_idx: Option<usize>,
    ) -> bool {
        let terrain = hill_idx.map(|idx| self.resources.terrain(idx));
        self.runner.render_hill_background(
            cx,
            terrain.as_deref(),
            Some(engine::color::Rgba::rgb(85, 85, 85)),
        )
    }

    pub fn update(&mut self, state: &mut GameState) {
        if self.is_save_dialog_active() || self.paused {
            return;
        }
        if self.runner.participant_is_computer() && self.phase() == Some(JumpPhase::Info) {
            self.runner.handle_input(JumpInput::LeaveInfo);
        }
        let previous_phase = self.runner.phase();
        let previous_takeoff = self.runner.state().map_or(0, |jump| jump.takeoff_counter);
        let is_human = !self.runner.participant_is_computer();
        self.runner.update(&mut state.rng, &mut state.wind);

        if previous_phase == Some(JumpPhase::OnBar)
            && self.runner.phase() == Some(JumpPhase::Disqualified)
        {
            state.request_beep(Beep::Type2);
        }
        let takeoff = self.runner.state().map_or(0, |jump| jump.takeoff_counter);
        if is_human && previous_takeoff < 17 && takeoff >= 17 {
            state.request_beep(Beep::Type1);
        }
    }

    fn build_runner(
        resources: ResourcesRef,
        state: &GameState,
        setup: RunnerSetup,
        snow: SnowSystem,
    ) -> JumpRunner {
        let RunnerSetup {
            hill_idx,
            start_gate,
            participant,
            policy,
            phase_label,
        } = setup;
        let hill = resources.hills.hill(hill_idx).map(|hill| hill.clone());
        let terrain = resources.terrain(hill_idx).as_ref().clone();
        let key = hill.as_ref().map(|h| &h.record_key);
        let record_distance = key
            .and_then(|k| state.records.hill_record(k))
            .map_or(0.0, |r| r.len);
        let goal_distance = goal_distance(state, &resources.hills, hill_idx);
        let snow_count = snow.count();
        let replay_competition_code =
            state
                .active_competition
                .as_ref()
                .and_then(|competition| match competition {
                    ActiveCompetition::Training => None,
                    ActiveCompetition::Individual(competition) => Some(match competition.style() {
                        CupStyle::WorldCup => 1,
                        CupStyle::CustomCup => 2,
                        CupStyle::FourHills => 3,
                        CupStyle::TeamCup => 4,
                    }),
                    ActiveCompetition::TeamCup(_) => Some(4),
                    ActiveCompetition::Koth(_) => Some(5),
                });
        JumpRunner::new(
            JumpConfig {
                hill_idx,
                is_custom_hill: hill_idx >= resources.hills.original_count(),
                hill,
                terrain,
                start_gate,
                snow_count,
                participant,
                policy,
                record_distance,
                goal_distance,
                phase_label,
                team_name: String::new(),
                replay_competition_code,
            },
            snow,
        )
    }
}

fn replay_default_author(participant: &JumpParticipant) -> String {
    participant.display_name().to_string()
}

fn allows_initial_custom_cup_wind_reset(state: &GameState, runner: &JumpRunner) -> bool {
    state.active_competition.as_ref().is_some_and(|active| {
        active.individual().is_some_and(|competition| {
            initial_custom_cup_wind_reset_context(
                competition.style(),
                competition.current_event,
                competition.phase(),
                competition.current_start_order_pos(),
                runner.participant_is_computer(),
            )
        })
    })
}

const fn initial_custom_cup_wind_reset_context(
    style: CupStyle,
    event: usize,
    phase: CompetitionPhase,
    start_order_pos: usize,
    is_computer: bool,
) -> bool {
    !is_computer
        && matches!(style, CupStyle::CustomCup)
        && event == 0
        && matches!(phase, CompetitionPhase::Round1)
        && start_order_pos == 0
}

fn is_localized_setup_key(event: UiEvent, localized_key: &str) -> bool {
    matches!(event, UiEvent::Text(c) if localized_key.chars().next().is_some_and(|key| c.eq_ignore_ascii_case(&key)))
}

fn is_localized_prompt_key(event: UiEvent, prompt: &str) -> bool {
    let Some(key) = prompt
        .strip_prefix('(')
        .and_then(|text| text.chars().next())
    else {
        return false;
    };
    matches!(event, UiEvent::Text(c) if c.eq_ignore_ascii_case(&key))
}

fn goal_distance(state: &GameState, hills: &HillCatalog, hill_idx: usize) -> f64 {
    if state.config.goals_enabled == 0 {
        return 0.0;
    }
    hills
        .hill(hill_idx)
        .and_then(|h| state.records.hill_goal(&h.record_key))
        .copied()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::{
        coach_telemetry, initial_custom_cup_wind_reset_context, is_localized_prompt_key,
        is_localized_setup_key, replay_default_author, JumpScene,
    };
    use crate::competition::types::{CompetitionPhase, CupStyle};
    use crate::gfx::color::Rgb6;
    use crate::jump::config::JumpParticipant;
    use crate::jump::policy::{JumpPolicy, JumperControl};
    use crate::rng::Random;
    use crate::store::GameState;
    use crate::ui::UiEvent;

    #[test]
    fn coach_telemetry_uses_jump_height_and_normalized_body_angle() {
        let telemetry = coach_telemetry(100, 63, 17, 580);

        assert_eq!(telemetry.height, 63);
        assert_eq!(telemetry.body_angle, 58);
        assert_eq!(telemetry.takeoff_timing, 17);
    }

    #[test]
    fn every_training_scene_replaces_stale_weather() {
        let mut state = GameState::default();
        state.first_event = false;
        state.rng = Random::new(5489);
        state.wind.strength = 999;
        state.wind.windy = 999;
        state.wind.value = 999;

        let first_snow = JumpScene::prepare_snow(&mut state, JumpPolicy::training());
        let first_weather = (state.wind.strength, state.wind.windy, state.wind.value);
        assert_ne!(first_weather, (999, 999, 999));

        state.wind.strength = 999;
        state.wind.windy = 999;
        state.wind.value = 999;
        let second_snow = JumpScene::prepare_snow(&mut state, JumpPolicy::training());
        let second_weather = (state.wind.strength, state.wind.windy, state.wind.value);

        assert_ne!(second_weather, (999, 999, 999));
        assert_ne!(
            (second_snow.count(), second_weather),
            (first_snow.count(), first_weather)
        );
    }

    #[test]
    fn low_detail_training_still_generates_weather_but_hides_snow() {
        let mut state = GameState::default();
        state.rng = Random::new(5489);
        state.config.graphics_detail = 1;
        state.wind.strength = 999;

        let snow = JumpScene::prepare_snow(&mut state, JumpPolicy::training());

        assert_eq!(snow.count(), 0);
        assert_ne!(state.wind.strength, 999);
    }

    #[test]
    fn setup_key_uses_localized_langbase_character() {
        assert!(is_localized_setup_key(UiEvent::Text('o'), "O"));
        assert!(is_localized_setup_key(UiEvent::Text('S'), "s"));
        assert!(!is_localized_setup_key(UiEvent::Text('s'), "O"));
    }

    #[test]
    fn replay_key_uses_the_localized_prompt() {
        assert!(is_localized_prompt_key(
            UiEvent::Text('z'),
            "(Z)apisz powtorke"
        ));
        assert!(is_localized_prompt_key(UiEvent::Text('S'), "(S)ave Replay"));
        assert!(!is_localized_prompt_key(
            UiEvent::Text('s'),
            "(Z)apisz powtorke"
        ));
    }

    #[test]
    fn replay_author_comes_from_actual_jump_participant() {
        let participant = JumpParticipant {
            id: 7,
            ai_id: 7,
            name: "PROFILE TWO".to_string(),
            real_name: "Actual Jumper".to_string(),
            suit_color: Rgb6([0; 3]),
            ski_color: Rgb6([0; 3]),
            team: None,
            control: JumperControl::Human,
        };

        assert_eq!(replay_default_author(&participant), "Actual Jumper");
    }

    #[test]
    fn f5_exception_is_only_first_human_of_initial_custom_cup_round_one() {
        assert!(initial_custom_cup_wind_reset_context(
            CupStyle::CustomCup,
            0,
            CompetitionPhase::Round1,
            0,
            false,
        ));
        assert!(!initial_custom_cup_wind_reset_context(
            CupStyle::CustomCup,
            1,
            CompetitionPhase::Round1,
            0,
            false,
        ));
        assert!(!initial_custom_cup_wind_reset_context(
            CupStyle::CustomCup,
            0,
            CompetitionPhase::Qualification,
            0,
            false,
        ));
        assert!(!initial_custom_cup_wind_reset_context(
            CupStyle::CustomCup,
            0,
            CompetitionPhase::Round1,
            1,
            false,
        ));
        assert!(!initial_custom_cup_wind_reset_context(
            CupStyle::CustomCup,
            0,
            CompetitionPhase::Round1,
            0,
            true,
        ));
        assert!(!initial_custom_cup_wind_reset_context(
            CupStyle::WorldCup,
            0,
            CompetitionPhase::Round1,
            0,
            false,
        ));
    }
}
