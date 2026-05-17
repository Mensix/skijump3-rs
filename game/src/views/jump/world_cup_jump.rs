use crate::competition::types::CompetitionPhase;
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{WorldCupCommand, WorldCupFlow};
use crate::gfx::palette::apply_menu_tint;
use crate::jump::JumpParticipant;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Jump,
    Results,
    Done,
}

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    controller: WorldCupFlow,
    display_page: Cell<usize>,
    render_mode: Cell<RenderMode>,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        JumpScene::setup_event(&store);
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            StoreRef::clone(&store),
            0,
            15,
            JumpParticipant::trainee(),
            crate::jump::JumpPolicy::competition(),
        );
        Self {
            resources,
            store,
            scene,
            controller: WorldCupFlow::new(),
            display_page: Cell::new(0),
            render_mode: Cell::new(RenderMode::Jump),
        }
    }

    fn record_finished_human_jump(&self) {
        let Some(outcome) = self.scene.outcome() else {
            return;
        };
        let comp = self.store.competition.borrow();
        let is_human = comp.as_ref().is_some_and(|c| c.is_human_current());
        drop(comp);
        if !is_human {
            return;
        }
        let mut comp = self.store.competition.borrow_mut();
        if let Some(c) = comp.as_mut() {
            c.record_jump(outcome.score, outcome.distance);
        }
    }

    fn handle_human_jump(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = self.scene.participant_id() != participant.id
            || self.scene.hill_idx() != hill_idx
            || self.scene.outcome().is_some();
        if needs_rebuild {
            self.controller.note_event_change(&self.store);
            self.scene
                .rebuild_for_competition(hill_idx, 15, participant, phase_label);
        } else {
            self.scene.set_phase_label(phase_label);
        }
    }

    fn results_page(&self) -> Vec<Element> {
        let comp = self.store.competition.borrow();
        let Some(c) = comp.as_ref() else {
            return vec![Element::fillbox(0, 0, 320, 200, 0)];
        };
        let page_data = competition_results::build_results_page(c, self.display_page.get());
        let mut els = competition_results::render_results_page(&page_data, &self.resources);
        els.extend(competition_results::render_header(c, &self.resources));
        els
    }

    fn drive_competition(&self) {
        let mut simulate_computer =
            |participant: JumpParticipant, hill_idx: usize| -> crate::jump::types::JumpOutcome {
                self.scene.simulate_hidden(participant, hill_idx)
            };

        let command = self
            .controller
            .drive(&self.resources, &self.store, &mut simulate_computer);

        match command {
            WorldCupCommand::HumanJump {
                participant,
                hill_idx,
                phase_label,
            } => {
                self.handle_human_jump(participant, hill_idx, phase_label);
                self.render_mode.set(RenderMode::Jump);
            }
            WorldCupCommand::ShowResults => {
                self.render_mode.set(RenderMode::Results);
            }
            WorldCupCommand::Done => {
                self.render_mode.set(RenderMode::Done);
            }
        }
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        self.record_finished_human_jump();
        self.drive_competition();
    }

    fn elements(&self) -> Vec<Element> {
        match self.render_mode.get() {
            RenderMode::Jump => self.scene.elements(),
            RenderMode::Results => self.results_page(),
            RenderMode::Done => vec![],
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.is_result_display_state() {
            match event {
                Event::Keyboard(Key::Right | Key::Char(' ')) => {
                    let page = self.display_page.get();
                    let total = {
                        let comp = self.store.competition.borrow();
                        let c = comp.as_ref()?;
                        competition_results::total_pages(c)
                    };
                    if page + 1 < total {
                        self.display_page.set(page + 1);
                    }
                    None
                }
                Event::Keyboard(Key::Left) => {
                    let page = self.display_page.get();
                    if page > 0 {
                        self.display_page.set(page - 1);
                    }
                    None
                }
                Event::Keyboard(Key::Escape | Key::Enter) => {
                    self.display_page.set(0);
                    self.controller.dismiss_display(&self.store);
                    None
                }
                _ => None,
            }
        } else {
            let action = {
                let mut session = self.scene.session_mut();
                JumpInputController.handle_event(event, &mut session)
            };
            match action {
                JumpInputAction::RouteBack => Some(RouteTarget::Back),
                _ => None,
            }
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        self.scene.render_snow(framebuffer);
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if self.is_result_display_state() {
            apply_menu_tint(palette, 3, 0);
            return;
        }
        self.scene.apply_palette(palette);
    }
}

impl WorldCupJumpView {
    fn is_result_display_state(&self) -> bool {
        if self.display_page.get() > 0 {
            return true;
        }
        self.store.competition.borrow().as_ref().is_some_and(|c| {
            matches!(
                c.phase,
                CompetitionPhase::QualificationResults
                    | CompetitionPhase::Round1Results
                    | CompetitionPhase::Round2Results
                    | CompetitionPhase::WorldCupStandings
                    | CompetitionPhase::SeasonComplete
            ) || matches!(
                c.phase,
                CompetitionPhase::Qualification
                    | CompetitionPhase::Round1
                    | CompetitionPhase::Round2
            ) && c.current_jumper().is_none()
        })
    }
}
