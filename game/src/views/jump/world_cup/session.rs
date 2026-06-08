use std::cell::Cell;

use super::flow::{self, WorldCupCommand};
use crate::competition::machine::Competition;
use crate::competition::scoring::wc_points_for_rank;
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::jump::types::JumpOutcome;
use crate::jump::JumpParticipant;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_tenths;
use crate::views::jump::scene::JumpScene;
use crate::views::jump::scene::JumpSceneError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum WorldCupSessionError {
    #[error("AI simulation failed: {0}")]
    JumpScene(#[from] JumpSceneError),
}

/// Describes what the view needs to set up for the next human jump.
#[derive(Debug, Clone)]
pub(crate) struct HumanJumpRequest {
    pub(crate) participant: JumpParticipant,
    pub(crate) hill_idx: usize,
    pub(crate) phase: CompetitionPhase,
    pub(crate) is_new_event: bool,
}

#[derive(Debug)]
pub(crate) enum WorldCupUiCommand {
    /// Prepare the jump scene for the next human jumper.
    HumanJump(HumanJumpRequest),
    /// Display the results/standings screen.
    ShowResults,
    /// Season batch is complete — persist results.
    Done,
}

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

    /// Record a human jump outcome into the competition, applying
    /// domain rules (crash → injury). Returns `true` if outcome was
    /// actually recorded (caller should then mark it as such in UI).
    pub(crate) fn record_finished_human_jump(&self, scene: &JumpScene) -> bool {
        let outcome = scene.outcome();
        let outcome = match outcome {
            Some(o) => o,
            None => return false,
        };
        if !self
            .store
            .try_with_competition(Competition::is_human_current)
            .unwrap_or(false)
        {
            return false;
        }
        self.store.try_with_competition_mut(|c| {
            c.apply_jump_outcome(outcome);
        });
        true
    }

    /// Drive the competition state machine forward.
    /// Returns a command the view should apply, or `None` if no
    /// competition is running. Returns `Err` if AI simulation fails
    /// (e.g. missing terrain/hill).
    pub(crate) fn drive_competition(
        &self,
        scene: &JumpScene,
    ) -> Result<Option<WorldCupUiCommand>, WorldCupSessionError> {
        let command = self.store.try_with_competition_mut(|c| {
            let mut simulate_computer = |participant: JumpParticipant,
                                         hill_idx: usize|
             -> Result<JumpOutcome, JumpSceneError> {
                scene.simulate_hidden(participant, hill_idx)
            };
            flow::drive(c, &self.last_event, &mut simulate_computer)
        });

        let command = match command {
            Some(cmd) => cmd?,
            None => return Ok(None),
        };

        Ok(Some(match command {
            WorldCupCommand::HumanJump {
                participant,
                hill_idx,
                phase,
                is_new_event,
            } => WorldCupUiCommand::HumanJump(HumanJumpRequest {
                participant,
                hill_idx,
                phase,
                is_new_event,
            }),
            WorldCupCommand::ShowResults => WorldCupUiCommand::ShowResults,
            WorldCupCommand::Done => {
                self.save_competition_results();
                WorldCupUiCommand::Done
            }
        }))
    }

    /// Persist competition results (profiles and records) to disk.
    /// Called on Done or SeasonComplete. Idempotent — runs once.
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
}

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
