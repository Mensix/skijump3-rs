use std::cell::Cell;

use crate::competition::runtime::CompetitionRuntime;
use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind};
use crate::jump::config::JumpParticipant;
use crate::jump::types::JumpOutcome;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::flow as competition_flow;
use crate::views::jump::competition::persistence;
use crate::views::jump::scene::JumpScene;
use crate::views::jump::scene::JumpSceneError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum TeamCupSessionError {
    #[error("AI simulation failed: {0}")]
    JumpScene(#[from] JumpSceneError),

    #[error("No Team Cup runtime in store")]
    NoRuntime,
}

#[derive(Debug, Clone)]
pub(crate) struct HumanJumpRequest {
    pub(crate) participant: JumpParticipant,
    pub(crate) hill_idx: usize,
    pub(crate) context: TeamCupJumpContext,
}

#[derive(Debug)]
pub(crate) enum TeamCupUiCommand {
    HumanJump(HumanJumpRequest),
    ShowResults(TeamCupResultsKind),
    Done,
}

#[derive(Debug)]
pub(crate) struct TeamCupSessionController {
    resources: ResourcesRef,
    store: StoreRef,
    profiles_saved: Cell<bool>,
}

impl TeamCupSessionController {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            resources,
            store,
            profiles_saved: Cell::new(false),
        }
    }

    pub(crate) fn record_finished_human_jump(&self, scene: &JumpScene) -> bool {
        let outcome = match scene.outcome() {
            Some(o) => o,
            None => return false,
        };
        self.store
            .try_with_team_cup_mut(|tc| {
                if !tc.is_human_current() {
                    return false;
                }
                let ctx = tc.current_jump_context();
                tc.record_jump_runtime(&ctx, outcome);
                true
            })
            .unwrap_or(false)
    }

    pub(crate) fn drive_competition(
        &self,
        scene: &JumpScene,
    ) -> Result<Option<TeamCupUiCommand>, TeamCupSessionError> {
        let command = self.store.try_with_team_cup_mut(|tc| {
            let mut simulate_computer = |participant: JumpParticipant,
                                         hill_idx: usize|
             -> Result<JumpOutcome, JumpSceneError> {
                scene.simulate_hidden(participant, hill_idx)
            };
            let mut mark_new_event = |_ctx: &TeamCupJumpContext, _flag: bool| false;
            competition_flow::drive(tc, &mut simulate_computer, &mut mark_new_event)
        });

        let command = match command {
            Some(cmd) => cmd?,
            None => return Ok(None),
        };

        Ok(Some(match command {
            competition_flow::CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => TeamCupUiCommand::HumanJump(HumanJumpRequest {
                participant,
                hill_idx,
                context,
            }),
            competition_flow::CompetitionFlowCommand::ShowResults(kind) => {
                TeamCupUiCommand::ShowResults(kind)
            }
            competition_flow::CompetitionFlowCommand::Done => {
                self.save_team_cup_results();
                TeamCupUiCommand::Done
            }
        }))
    }

    pub(crate) fn save_team_cup_results(&self) {
        persistence::save_profiles_once(&self.profiles_saved, &self.resources, &self.store);
    }
}
