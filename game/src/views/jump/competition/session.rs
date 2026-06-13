use std::cell::Cell;

use crate::competition::runtime::CompetitionRuntime;
use crate::jump::types::JumpOutcome;
use crate::jump::JumpParticipant;
use crate::store::{HasRuntime, ResourcesRef, Store, StoreRef};
use crate::views::jump::competition::flow::{self, CompetitionFlowCommand};
use crate::views::jump::competition::persistence;
use crate::views::jump::scene::{JumpScene, JumpSceneError};

#[derive(Debug, thiserror::Error)]
pub(crate) enum SessionError {
    #[error("AI simulation failed: {0}")]
    JumpScene(#[from] JumpSceneError),
}

#[derive(Debug)]
pub(crate) struct CompetitionSession {
    resources: ResourcesRef,
    store: StoreRef,
    last_event: Cell<usize>,
    profiles_saved: Cell<bool>,
}

impl CompetitionSession {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            resources,
            store,
            last_event: Cell::new(0),
            profiles_saved: Cell::new(false),
        }
    }

    pub(crate) fn record_finished_human_jump<R>(&self, scene: &JumpScene) -> bool
    where
        R: CompetitionRuntime + 'static,
        Store: HasRuntime<R>,
    {
        let outcome = match scene.outcome() {
            Some(o) => o,
            None => return false,
        };
        scene.collect_telemetry();
        self.store
            .with_runtime_mut(|r: &mut R| {
                if !r.is_human_current() {
                    return false;
                }
                let ctx = r.current_jump_context();
                r.record_jump_runtime(&ctx, outcome);
                true
            })
            .unwrap_or(false)
    }

    pub(crate) fn drive_competition<R>(
        &self,
        scene: &JumpScene,
    ) -> Result<Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>, SessionError>
    where
        R: CompetitionRuntime + 'static,
        Store: HasRuntime<R>,
    {
        let command = self.store.with_runtime_mut(|r: &mut R| {
            let current_event = r.event_idx();
            let mut simulate_computer = |participant: JumpParticipant,
                                         hill_idx: usize|
             -> Result<JumpOutcome, JumpSceneError> {
                scene.simulate_hidden(participant, hill_idx)
            };
            let mut mark_new_event =
                |_: &R::Context, _: bool| check_event_change(current_event, &self.last_event);
            flow::drive(r, &mut simulate_computer, &mut mark_new_event)
        });

        let command = match command {
            Some(cmd) => cmd?,
            None => return Ok(None),
        };

        Ok(Some(command))
    }

    pub(crate) fn advance_results_and_drive<R>(
        &self,
        scene: &JumpScene,
    ) -> Result<Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>, SessionError>
    where
        R: CompetitionRuntime + 'static,
        Store: HasRuntime<R>,
    {
        self.store.with_runtime_mut(|r: &mut R| {
            r.advance_results_runtime();
        });
        self.drive_competition::<R>(scene)
    }

    pub(crate) fn save_results(&self) {
        if self.profiles_saved.get() {
            return;
        }
        persistence::save_profiles_and_records_once(
            &self.profiles_saved,
            &self.resources,
            &self.store,
        );
    }
}

fn check_event_change(current: usize, last_event: &Cell<usize>) -> bool {
    let new = current != last_event.get();
    if new {
        last_event.set(current);
    }
    new
}
