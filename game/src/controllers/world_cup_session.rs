use std::cell::Cell;

use crate::competition::machine::Competition;
use crate::competition::scoring::wc_points_for_rank;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::controllers::competition_ui::{CompetitionUiState, RenderMode};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
use crate::jump::types::{FallType, JumpOutcome};
use crate::jump::JumpParticipant;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_tenths;

#[derive(Debug)]
pub(crate) struct WorldCupSessionController {
    resources: ResourcesRef,
    store: StoreRef,
    last_event: Cell<usize>,
    profiles_saved: Cell<bool>,
}

impl WorldCupSessionController {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            resources,
            store,
            last_event: Cell::new(0),
            profiles_saved: Cell::new(false),
        }
    }

    pub(crate) fn record_finished_human_jump(
        &self,
        scene: &JumpScene,
        ui_state: &CompetitionUiState,
    ) {
        let outcome = scene.outcome();
        if outcome.is_none() || !ui_state.is_result_acknowledged() || ui_state.is_outcome_recorded()
        {
            return;
        }
        let outcome = outcome.unwrap();
        if !self
            .store
            .try_with_competition(Competition::is_human_current)
            .unwrap_or(false)
        {
            return;
        }
        self.store.try_with_competition_mut(|c| {
            if outcome.fall_type == FallType::Crash {
                c.injure_current(3);
            }
            c.record_jump(outcome.score, outcome.distance);
        });
        ui_state.mark_outcome_recorded();
    }

    pub(crate) fn drive_competition(&self, scene: &JumpScene, ui_state: &CompetitionUiState) {
        let command = self.store.try_with_competition_mut(|c| {
            let mut simulate_computer =
                |participant: JumpParticipant, hill_idx: usize| -> JumpOutcome {
                    scene.simulate_hidden(participant, hill_idx)
                };
            world_cup_flow::drive(c, &self.last_event, &mut simulate_computer)
        });

        let Some(command) = command else {
            ui_state.enter_done();
            return;
        };

        match command {
            WorldCupCommand::HumanJump {
                participant,
                hill_idx,
                phase,
                is_new_event,
            } => {
                if is_new_event {
                    JumpScene::setup_event(&self.store);
                }
                let phase_label = Self::phase_label(&self.resources, phase);
                self.handle_human_jump(scene, ui_state, participant, hill_idx, phase_label);
                ui_state.enter_jump();
            }
            WorldCupCommand::ShowResults => {
                // Only initialize result UI on first entry, otherwise paging/toggles reset.
                if ui_state.render_mode() != RenderMode::Results {
                    self.select_default_result_screen(ui_state);
                    ui_state.enter_results();
                }
            }
            WorldCupCommand::Done => {
                self.save_competition_results();
                ui_state.enter_done();
            }
        }
    }

    pub(crate) fn save_competition_results(&self) {
        if self.profiles_saved.replace(true) {
            return;
        }
        self.store.try_with_competition(|c| {
            let style = c.style();
            let overall = c.overall_standings();
            let mut profiles = self.store.profiles_mut();

            for p in &overall {
                let Some(pidx) = p.profile_idx else {
                    continue;
                };
                let Some(profile) = profiles.profiles.get_mut(pidx) else {
                    continue;
                };

                match style {
                    CupStyle::WorldCup => {
                        profile.world_cups += 1;
                        let my_points = p.points.unwrap_or(0);
                        let event_rank = event_rank_by_points(c, my_points);
                        let pts = wc_points_for_rank(event_rank);
                        if pts >= profile.bestpoints as i32 {
                            profile.bestpoints = pts as usize;
                            profile.best_result = format_wc_best_result(pts, event_rank);
                        }
                        if p.four_hills_points > 0
                            && p.four_hills_points >= profile.best4points as i32
                        {
                            profile.best4points = p.four_hills_points as usize;
                            profile.best_4h_result =
                                format_four_hills_best_result(p.four_hills_points, p.rank);
                        }
                    }
                    CupStyle::FourHills => {
                        if p.four_hills_points >= profile.best4points as i32 {
                            profile.best4points = p.four_hills_points as usize;
                            profile.best_4h_result =
                                format_four_hills_best_result(p.four_hills_points, p.rank);
                        }
                    }
                    CupStyle::CustomCup | CupStyle::TeamCup => {}
                }
            }
        });
        if let Err(e) = self
            .resources
            .save_manager
            .save_players(&self.store.profiles())
        {
            eprintln!("Warning: failed to save players: {e}");
        }
        if let Some(records) = self.store.try_records() {
            if let Err(e) = self.resources.save_manager.save_records(&records) {
                eprintln!("Warning: failed to save records: {e}");
            }
        }
    }

    fn handle_human_jump(
        &self,
        scene: &JumpScene,
        ui_state: &CompetitionUiState,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = scene.participant_id() != participant.id
            || scene.hill_idx() != hill_idx
            || ui_state.is_outcome_recorded()
            || (scene.outcome().is_some() && ui_state.is_result_acknowledged());
        if needs_rebuild {
            ui_state.reset_acknowledged();
            ui_state.reset_outcome_recorded();
            scene.rebuild_for_competition(hill_idx, 15, participant, phase_label);
        } else {
            scene.set_phase_label(phase_label);
        }
    }

    fn select_default_result_screen(&self, ui_state: &CompetitionUiState) {
        if let Some(phase) = self.store.try_with_competition(Competition::phase) {
            let is_4h = self
                .store
                .try_with_competition(Competition::is_four_hills_event)
                .unwrap_or(false);
            ui_state.select_default_screen(is_4h, phase);
        }
    }

    fn phase_label(resources: &ResourcesRef, phase: CompetitionPhase) -> String {
        match phase {
            CompetitionPhase::Training(n) => format!("{} {}", resources.langbase.lstr(52), n),
            CompetitionPhase::Qualification => resources.langbase.lstr(53).to_string(),
            CompetitionPhase::Round1 => resources.langbase.lstr(54).to_string(),
            CompetitionPhase::Round2 => resources.langbase.lstr(55).to_string(),
            _ => resources.langbase.lstr(51).to_string(),
        }
    }
}

/// Pascal: `txt(mcpisteet[who])+' ('+str1+')'` where str1 is `sija[who]+'.'`
/// for the final event. Same applies to best4 result with `txtp` for tenths.
fn format_wc_best_result(points: i32, rank: usize) -> String {
    format!("{points} ({rank}.)")
}

fn format_four_hills_best_result(points_tenths: i32, rank: usize) -> String {
    format!("{} ({}.)", format_tenths(points_tenths), rank)
}

/// Tie-aware event rank: 1 + count of participants with strictly higher points.
fn event_rank_by_points(competition: &Competition, points: i32) -> usize {
    1 + competition
        .event_standings()
        .iter()
        .filter(|p| p.points.unwrap_or(i32::MIN) > points)
        .count()
}
