use super::results;
use crate::competition::completion::CompletionSaveGuard;
use crate::competition::machine::Competition;
use crate::competition::runtime::{IndividualJumpContext, IndividualResultsKind};
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::components::modal::{ConfirmationChoice, Modal, ModalResult};
use crate::components::page_nav::{
    is_quit_event, page_event, PageDismissal, PageEvent, PageEventMap,
};
use crate::data::profile::Profile;
use crate::data::records::{insert_ranked_hiscore, Hiscore};
use crate::jump::types::JumpPhase;
use crate::route::RouteTarget;
use crate::save::custom_cup::{load_custom_cup_file, save_custom_cup_file};
use crate::screen::{GameCx, GameScreen, NavSignal, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::current_timestamp;
use crate::text::format::format_decimal;
use crate::ui::UiCanvas;
use crate::ui::{Blinker, Key, ScreenEventCx, UiEvent};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::results as competition_results;
use crate::views::jump::competition::ui_state::{RenderMode, ResultScreen};
use crate::views::records::RecordNotification;

pub struct WorldCupJumpView {
    controller: CompetitionJumpController<Competition>,
    blinker: Blinker,
    completion_saved: CompletionSaveGuard,
    record_notification: Option<RecordNotification>,
    quit_modal: Option<Modal>,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, state: &GameState) -> Self {
        Self {
            controller: CompetitionJumpController::new(
                resources,
                state.config.compact_results != 0,
                None,
            ),
            blinker: Blinker::new(),
            completion_saved: CompletionSaveGuard::default(),
            record_notification: None,
            quit_modal: None,
        }
    }

    fn begin_quit_confirmation(&mut self, cx: &mut GameCx<'_>) {
        let prompt = 256 + (cx.state.rng.random_i32(3) as usize).min(2);
        let line1 = cx.layout.langbase.tr(245).to_owned();
        let line2 = cx.layout.langbase.tr(prompt).to_owned();
        self.quit_modal = Some(Modal::confirm(line1, line2));
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<IndividualJumpContext, IndividualResultsKind>,
        state: &mut GameState,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
            } => {
                let phase_label = phase_label(self.controller.resources(), context.phase);
                let is_leader = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| {
                        active.individual().and_then(|c| {
                            c.overall_standings()
                                .first()
                                .map(|s| s.id == participant.id)
                        })
                    })
                    .unwrap_or(false);
                self.controller.prepare_human_jump(
                    participant,
                    hill_idx,
                    phase_label,
                    None,
                    !matches!(context.phase, CompetitionPhase::Training(_)),
                    state,
                );
                if let Some(scene) = self.controller.scene_mut() {
                    scene.set_has_bib(is_leader);
                }
            }
            CompetitionFlowCommand::ShowResults(IndividualResultsKind::Results) => {
                if self.controller.render_mode() != RenderMode::Results {
                    self.select_default_result_screen(state);
                    self.controller.enter_results();
                }
            }
            CompetitionFlowCommand::Done => {
                self.controller.enter_done();
            }
        }
    }

    fn select_default_result_screen(&mut self, state: &GameState) {
        if let Some(phase) = state
            .active_competition
            .as_ref()
            .and_then(|active| active.individual().map(Competition::phase))
        {
            let is_4h = state
                .active_competition
                .as_ref()
                .and_then(|active| active.individual().map(Competition::is_four_hills_event))
                .unwrap_or(false);
            self.controller
                .ui_state_mut()
                .select_default_screen(is_4h, phase);
            let extra_stats_enabled = state.config.extra_statistics != 0;
            let showing_ko_pairs = is_4h
                && matches!(
                    phase,
                    CompetitionPhase::QualificationResults | CompetitionPhase::Round1Results
                );
            if extra_stats_enabled && !showing_ko_pairs {
                let show_stats = matches!(
                    phase,
                    CompetitionPhase::FourHillsStandings
                        | CompetitionPhase::WorldCupStandings
                        | CompetitionPhase::SeasonComplete
                );
                if show_stats
                    && state
                        .active_competition
                        .as_ref()
                        .and_then(|active| active.individual().map(results::stats_total_pages))
                        .is_some_and(|pages| pages > 0)
                {
                    self.controller.ui_state_mut().select_stats_screen();
                }
            }
        }
    }

    fn results_page(&self, cx: &mut dyn UiCanvas, state: &GameState) {
        let hill_background = self.controller.render_results_background(cx, state);
        competition_results::render(
            cx,
            self.controller.resources(),
            state,
            self.controller.ui_state(),
            self.blinker.visible(10, 10),
            hill_background,
        )
    }

    fn paint_content(&mut self, cx: &mut dyn UiCanvas, state: &GameState) {
        if self.controller.paint_alerts(&self.record_notification, cx) {
            return;
        }
        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
            }
            RenderMode::Results => self.results_page(cx, state),
            RenderMode::Done => {}
        }
    }

    fn handle_input(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> NavSignal {
        if let Some(signal) = self
            .controller
            .handle_alert_input(&mut self.record_notification, event)
        {
            return signal;
        }
        if self.is_result_display_state(state) {
            return self.handle_result_event(event, state, cx);
        }

        let is_dq =
            self.controller.scene().and_then(|s| s.phase()) == Some(JumpPhase::Disqualified);
        match self
            .controller
            .handle_jump_scene_event(event, true, !is_dq, false, state, cx)
        {
            JumpInputResult::Consumed => return NavSignal::None,
            JumpInputResult::OpenSetup => return NavSignal::Route(RouteTarget::OptionsMenu),
            JumpInputResult::None => {}
        }

        NavSignal::None
    }
}

impl GameScreen for WorldCupJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        if self.controller.has_profile_alert() {
            return;
        }
        let persistence = cx.persistence();
        self.controller
            .record_acknowledged_human_jump(cx.state, &persistence);

        if let Some(command) = self.controller.drive(cx.state) {
            self.apply_command(command, cx.state);
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene(cx.state);
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.controller.has_profile_alert() {
            match event {
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(_, _) => {
                    nav.back();
                }
                UiEvent::Quit | UiEvent::Tick => {}
            }
            nav.consume();
            return;
        }
        if let Some(modal) = &self.quit_modal {
            match modal.event(event, &cx.layout.langbase) {
                Some(ModalResult::Confirmed(ConfirmationChoice::Yes)) => {
                    self.quit_modal = None;
                    cx.state.abort_active_competition();
                    nav.navigate(RouteTarget::MainMenu);
                }
                Some(ModalResult::Confirmed(ConfirmationChoice::No))
                | Some(ModalResult::Dismissed) => {
                    self.quit_modal = None;
                    nav.consume();
                }
                None => nav.consume(),
            }
            return;
        }
        if self.controller.ui_state_mut().take_cup_exit_request()
            && matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_))
            && !self
                .controller
                .scene()
                .is_some_and(|scene| scene.is_save_dialog_active())
        {
            cx.state.abort_active_competition();
            nav.navigate(RouteTarget::MainMenu);
            return;
        }
        if is_quit_event(event) {
            if self.is_result_display_state(cx.state) {
                cx.state.abort_active_competition();
                nav.navigate(RouteTarget::MainMenu);
            } else {
                self.controller.ui_state_mut().request_cup_exit();
                let persistence = cx.persistence();
                self.handle_input(UiEvent::KeyDown(Key::Escape), cx.state, &persistence)
                    .dispatch(nav);
            }
            return;
        }
        if self.is_result_display_state(cx.state) {
            if matches!(event, UiEvent::KeyDown(Key::Escape)) {
                self.begin_quit_confirmation(cx);
                nav.consume();
            } else {
                let persistence = cx.persistence();
                self.handle_input(event, cx.state, &persistence)
                    .dispatch(nav);
            }
            return;
        }
        let persistence = cx.persistence();
        self.handle_input(event, cx.state, &persistence)
            .dispatch(nav);
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint, cx.state);
        if let Some(modal) = &self.quit_modal {
            modal.paint(paint, &cx.layout.langbase);
        }
    }

    fn has_modal(&self) -> bool {
        self.quit_modal.is_some() || self.controller.has_profile_alert()
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self, state: &GameState) -> bool {
        if self.controller.ui_state().has_page() {
            return true;
        }
        state
            .active_competition
            .as_ref()
            .and_then(|active| {
                let c = active.individual()?;
                Some(
                    c.phase().is_result_phase()
                        || c.phase().needs_event_results() && c.current_jumper().is_none(),
                )
            })
            .unwrap_or(false)
    }

    fn save_competition_results(&mut self, state: &mut GameState, cx: &Persistence) {
        if !self.completion_saved.claim() {
            return;
        }
        let custom_notification = self.update_custom_cup_records(state);
        let hall_notification = self.update_hall_of_fame(state);
        self.record_notification = custom_notification.or(hall_notification);

        let (style, participants): (_, Vec<_>) = {
            let active = match state.active_competition.as_ref() {
                Some(a) => a,
                None => return,
            };
            let Some(c) = active.individual() else {
                return;
            };
            let style = c.style();
            let standings = c.overall_standings();
            let participants = standings
                .iter()
                .map(|p| {
                    (
                        p.profile_idx,
                        p.wc_points,
                        p.four_hills_points,
                        p.rank,
                        rank_for_score(
                            p.four_hills_points,
                            standings.iter().map(|other| other.four_hills_points),
                        ),
                        p.leg_wins,
                    )
                })
                .collect();
            (style, participants)
        };
        let profiles = &mut state.profiles;
        for (pos, &(pidx_opt, wc_points, fh_points, rank, fh_rank, leg_wins)) in
            participants.iter().enumerate()
        {
            let Some(pidx) = pidx_opt else { continue };
            let Some(profile) = profiles.profiles.get_mut(pidx) else {
                continue;
            };
            match style {
                CupStyle::WorldCup => {
                    profile.world_cups += 1;
                    profile.legs_won += leg_wins;
                    if pos == 0 {
                        profile.world_cups_won += 1;
                    }
                    update_world_cup_best(profile, wc_points, rank);
                    if fh_points > 0.0 && fh_points >= profile.best_4h_points {
                        profile.best_4h_points = fh_points;
                        profile.best_4h_result = format_four_hills_best_result(fh_points, fh_rank);
                    }
                }
                CupStyle::FourHills if fh_points >= profile.best_4h_points => {
                    profile.best_4h_points = fh_points;
                    profile.best_4h_result = format_four_hills_best_result(fh_points, fh_rank);
                }
                _ => {}
            }
        }
        self.controller.save_results(state, cx);
    }

    fn update_custom_cup_records(&self, state: &GameState) -> Option<RecordNotification> {
        let competition = state
            .active_competition
            .as_ref()
            .and_then(|active| active.individual())?;
        if competition.style() != CupStyle::CustomCup {
            return None;
        }
        let name = competition.custom_cup_file()?;
        let mut file = load_custom_cup_file(&self.controller.resources().files, name)?;

        let (inserted, old_record) =
            update_custom_records(&mut file.records, competition, &current_timestamp());
        save_custom_cup_file(&self.controller.resources().files, name, &file);
        (!inserted.is_empty()).then(|| {
            RecordNotification::new(
                self.controller.resources().langbase.tr(128),
                inserted,
                old_record,
            )
        })
    }

    fn update_hall_of_fame(&self, state: &mut GameState) -> Option<RecordNotification> {
        let (style, wc_results, fh_results) = {
            let active = state.active_competition.as_ref()?;
            let c = active.individual()?;
            let participants = c.overall_standings();
            let style = c.style();

            let wc_results: Vec<_> = participants
                .iter()
                .filter(|p| !p.is_computer)
                .map(|p| (p.display_name().to_string(), p.wc_points as f64, p.rank))
                .collect();

            let mut fh: Vec<_> = participants
                .iter()
                .filter(|p| p.four_hills_points > 0.0)
                .map(|p| {
                    (
                        p.display_name().to_string(),
                        p.four_hills_points,
                        p.is_computer,
                        0,
                    )
                })
                .collect();
            fh.sort_by(|a, b| b.1.total_cmp(&a.1));
            for i in 0..fh.len() {
                fh[i].3 = if i > 0 && fh[i].1 == fh[i - 1].1 {
                    fh[i - 1].3
                } else {
                    i + 1
                };
            }
            let fh_results: Vec<_> = fh
                .into_iter()
                .filter(|(_, _, is_computer, _)| !is_computer)
                .map(|(name, score, _, rank)| (name, score, rank))
                .collect();

            (style, wc_results, fh_results)
        };

        let time = current_timestamp();
        let mut inserted = Vec::new();

        match style {
            CupStyle::WorldCup => {
                for (name, score, rank) in &wc_results {
                    let candidate = Hiscore {
                        name: name.clone(),
                        pos: *rank,
                        score: *score,
                        time: time.clone(),
                        is_computer: false,
                    };
                    if insert_ranked_hiscore(&mut state.records.top, 0..20, candidate.clone()) {
                        inserted.push(candidate);
                    }
                }
                for (name, score, rank) in &fh_results {
                    let candidate = Hiscore {
                        name: name.clone(),
                        pos: *rank,
                        score: *score,
                        time: time.clone(),
                        is_computer: false,
                    };
                    if insert_ranked_hiscore(&mut state.records.top, 30..35, candidate.clone()) {
                        inserted.push(candidate);
                    }
                }
            }
            CupStyle::FourHills => {
                for (name, score, rank) in &fh_results {
                    let candidate = Hiscore {
                        name: name.clone(),
                        pos: *rank,
                        score: *score,
                        time: time.clone(),
                        is_computer: false,
                    };
                    if insert_ranked_hiscore(&mut state.records.top, 30..35, candidate.clone()) {
                        inserted.push(candidate);
                    }
                }
            }
            _ => {}
        }
        (!inserted.is_empty())
            .then(|| RecordNotification::new("NEW HALL OF FAME RECORD!", inserted, None))
    }

    fn dismiss_results_and_advance(&mut self, state: &mut GameState) {
        self.blinker.reset();
        if let Some(command) = self.controller.dismiss_results_and_advance(state) {
            self.apply_command(command, state);
        }
    }

    fn handle_result_event(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> NavSignal {
        if let Some(page_event) = page_event(event, PageEventMap::Individual) {
            let total = match self.controller.ui_state().current_screen() {
                ResultScreen::Stats => state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.individual().map(results::stats_total_pages))
                    .unwrap_or(1),
                ResultScreen::KoPairs(_) => 1,
                ResultScreen::List if self.controller.ui_state().is_compact() => 1,
                ResultScreen::List => state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.individual().map(results::total_pages))
                    .unwrap_or(1),
            };
            if self.controller.ui_state_mut().handle_page_event(
                page_event,
                total,
                PageDismissal::INDIVIDUAL,
            ) {
                let season_complete = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.individual())
                    .is_some_and(|c| c.phase() == CompetitionPhase::SeasonComplete);
                if page_event == PageEvent::Confirm && season_complete {
                    self.save_competition_results(state, cx);
                    return if self.record_notification.is_some() {
                        NavSignal::None
                    } else {
                        NavSignal::Back
                    };
                }
                self.dismiss_results_and_advance(state);
            }
            return NavSignal::None;
        }

        match event {
            UiEvent::TextWithModifiers('c' | 'C', modifiers) if modifiers.ctrl => {
                self.controller.ui_state_mut().toggle_compact();
                NavSignal::None
            }
            UiEvent::Text('c' | 'C') => {
                self.controller.ui_state_mut().toggle_compact();
                NavSignal::None
            }
            UiEvent::Text('s' | 'S') => {
                let has_stats = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.individual().map(results::stats_total_pages))
                    .is_some_and(|pages| pages > 0);
                if has_stats || self.controller.ui_state().current_screen() == ResultScreen::Stats {
                    self.controller.ui_state_mut().toggle_stats();
                }
                NavSignal::None
            }
            UiEvent::Text('k' | 'K') => {
                let ko = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| {
                        let c = active.individual()?;
                        Some(
                            c.is_four_hills_event()
                                && matches!(
                                    c.phase(),
                                    CompetitionPhase::QualificationResults
                                        | CompetitionPhase::Round1Results
                                ),
                        )
                    })
                    .unwrap_or(false);
                if ko {
                    let round1 = state
                        .active_competition
                        .as_ref()
                        .and_then(|active| {
                            active
                                .individual()
                                .map(|c| c.phase() == CompetitionPhase::Round1Results)
                        })
                        .unwrap_or(false);
                    self.controller.ui_state_mut().toggle_ko_pairs(round1);
                }
                NavSignal::None
            }
            _ => NavSignal::None,
        }
    }
}

fn update_custom_records(
    records: &mut Vec<Hiscore>,
    competition: &Competition,
    time: &str,
) -> (Vec<Hiscore>, Option<Hiscore>) {
    records.resize_with(20, Hiscore::default);
    let mut inserted = Vec::new();
    let mut old_record = None;
    for participant in competition
        .overall_standings()
        .into_iter()
        .filter(|participant| !participant.is_computer)
    {
        let score = if competition.uses_aggregate_standings() {
            participant.four_hills_points
        } else {
            f64::from(participant.wc_points)
        };
        let candidate = Hiscore {
            name: participant.display_name().to_string(),
            pos: participant.rank,
            score,
            time: time.to_string(),
            is_computer: false,
        };
        let previous = records
            .iter()
            .filter(|record| record.name == candidate.name)
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .cloned();
        if insert_ranked_hiscore(records, 0..20, candidate.clone()) {
            old_record = old_record.or(previous.filter(|record| !record.name.is_empty()));
            inserted.push(candidate);
        }
    }
    (inserted, old_record)
}

fn phase_label(resources: &ResourcesRef, phase: CompetitionPhase) -> String {
    let lang = &resources.langbase;
    match phase {
        CompetitionPhase::Training(n) => format!("{} {}", lang.tr(52), n),
        CompetitionPhase::Qualification => lang.tr(53).to_string(),
        CompetitionPhase::Round1 => lang.tr(54).to_string(),
        CompetitionPhase::Round2 => lang.tr(55).to_string(),
        _ => lang.tr(51).to_string(),
    }
}

fn format_wc_best_result(points: i32, rank: usize) -> String {
    format!("{points} ({rank}.)")
}

fn update_world_cup_best(profile: &mut Profile, points: i32, rank: usize) {
    if points >= profile.best_points as i32 {
        profile.best_points = points as usize;
        profile.best_result = format_wc_best_result(points, rank);
    }
}

fn format_four_hills_best_result(points: f64, rank: usize) -> String {
    format!("{} ({}.)", format_decimal(points), rank)
}

fn rank_for_score(scores: f64, field: impl Iterator<Item = f64>) -> usize {
    1 + field.filter(|&other| other > scores).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::field::SortBy;
    use crate::competition::types::{CustomCupScoring, Participant};

    #[test]
    fn world_cup_best_uses_season_points_and_final_rank() {
        let mut profile = Profile {
            best_points: 250,
            best_result: "250 (4.)".to_string(),
            ..Profile::default()
        };

        update_world_cup_best(&mut profile, 240, 2);
        assert_eq!(profile.best_points, 250);
        assert_eq!(profile.best_result, "250 (4.)");

        update_world_cup_best(&mut profile, 875, 3);
        assert_eq!(profile.best_points, 875);
        assert_eq!(profile.best_result, "875 (3.)");
    }

    #[test]
    fn custom_record_insertion_uses_mode_and_keeps_better_same_name_result() {
        let mut human = Participant::computer(0, 0, "PLAYER".into());
        human.is_computer = false;
        human.rank = 2;
        human.wc_points = 80;
        human.four_hills_points = 456.7;
        let mut competition = Competition::new(CupStyle::CustomCup, vec![human], vec![0]);
        competition.custom_cup_scoring = CustomCupScoring::WorldCupPoints;
        competition.field.sort_field(SortBy::WcPoints);
        let mut records = vec![Hiscore {
            name: "PLAYER".into(),
            pos: 1,
            score: 100.0,
            time: "OLD".into(),
            is_computer: false,
        }];

        let (inserted, old) = update_custom_records(&mut records, &competition, "NEW");

        assert_eq!(records.len(), 20);
        assert_eq!(records[0].score, 100.0);
        assert_eq!(records[0].time, "OLD");
        assert!(inserted.is_empty());
        assert!(old.is_none());

        competition.custom_cup_scoring = CustomCupScoring::AggregateJumpPoints;
        competition.field.sort_field(SortBy::FourHillsPoints);
        let (inserted, old) = update_custom_records(&mut records, &competition, "NEW");
        assert_eq!(records[0].score, 456.7);
        assert_eq!(records[0].time, "NEW");
        assert_eq!(inserted.len(), 1);
        assert_eq!(old.map(|record| record.score), Some(100.0));
    }

    #[test]
    fn four_hills_rank_is_independent_of_final_world_cup_order() {
        let scores = [900.0, 850.0, 800.0];
        assert_eq!(rank_for_score(850.0, scores.into_iter()), 2);
        assert_eq!(format_four_hills_best_result(850.0, 2), "850.0 (2.)");
    }
}
