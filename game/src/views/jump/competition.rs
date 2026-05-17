use crate::competition::types::CompetitionPhase;
use crate::jump::JumpRunner;
use crate::gfx::palette::apply_menu_tint;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::controllers::competition_jump::{
    CompetitionJumpController, CompetitionRenderState,
};
use crate::controllers::training_jump::{TrainingJumpAction, TrainingJumpController};
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::{Cell, RefCell};

pub(crate) struct CompetitionJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    runner: RefCell<JumpRunner>,
    controller: CompetitionJumpController,
    display_page: Cell<usize>,
}

impl CompetitionJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let controller = CompetitionJumpController::new();
        let runner = CompetitionJumpController::initial_runner(&resources, &store);
        Self {
            resources,
            store,
            runner: RefCell::new(runner),
            controller,
            display_page: Cell::new(0),
        }
    }
}

impl View<RouteTarget> for CompetitionJumpView {
    fn elements(&self) -> Vec<Element> {
        match self
            .controller
            .drive(&self.resources, &self.store, &self.runner)
        {
            CompetitionRenderState::HumanJump => self
                .runner
                .borrow_mut()
                .elements(&self.resources, &self.store),
            CompetitionRenderState::DisplayList => {
                let comp = self.store.competition.borrow();
                let Some(c) = comp.as_ref() else {
                    return vec![Element::fillbox(0, 0, 320, 200, 0)];
                };
                let page_data = competition_results::build_results_page(c, self.display_page.get());
                let mut els = competition_results::render_results_page(&page_data, &self.resources);
                els.extend(competition_results::render_header(c, &self.resources));
                els
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let comp_phase = self.store.competition.borrow().as_ref().map(|c| c.phase);

        if matches!(
            comp_phase,
            Some(
                CompetitionPhase::QualificationResults
                    | CompetitionPhase::Round1Results
                    | CompetitionPhase::Round2Results
                    | CompetitionPhase::WorldCupStandings
                    | CompetitionPhase::SeasonComplete
            )
        ) || self.display_page.get() > 0
            || {
                // Check if we're in a displayable jump phase with no current jumper
                self.store.competition.borrow().as_ref().is_some_and(|c| {
                    matches!(
                        c.phase,
                        CompetitionPhase::Qualification
                            | CompetitionPhase::Round1
                            | CompetitionPhase::Round2
                    ) && c.current_jumper().is_none()
                })
            }
        {
            // We're in a display phase — handle pagination
            match event {
                Event::Keyboard(Key::Right | Key::Char(' ')) => {
                    let page = self.display_page.get();
                    let total = {
                        let comp = self.store.competition.borrow();
                        let Some(c) = comp.as_ref() else {
                            return None;
                        };
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
            // Jump phase — delegate to TrainingJumpController
            let action = {
                let mut runner = self.runner.borrow_mut();
                TrainingJumpController.handle_event(event, runner.session_mut())
            };
            match action {
                TrainingJumpAction::RouteBack => Some(RouteTarget::Back),
                _ => None,
            }
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Ok(mut runner) = self.runner.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            runner.render_snow(framebuffer, wind);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if self.is_displaying_results() {
            apply_menu_tint(palette, 3, 0);
            return;
        }

        if let Ok(runner) = self.runner.try_borrow() {
            runner.apply_palette(palette);
        }
    }
}

impl CompetitionJumpView {
    fn is_displaying_results(&self) -> bool {
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
