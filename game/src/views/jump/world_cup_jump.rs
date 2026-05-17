use crate::competition::types::CompetitionPhase;
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{WorldCupFlow, WorldCupScreenState};
use crate::gfx::palette::apply_menu_tint;
use crate::jump::JumpParticipant;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::results as competition_results;
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::Cell;

pub struct WorldCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: JumpScene,
    controller: WorldCupFlow,
    display_page: Cell<usize>,
}

impl WorldCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
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
        }
    }
}

impl View<RouteTarget> for WorldCupJumpView {
    fn elements(&self) -> Vec<Element> {
        match self
            .controller
            .drive(&self.resources, &self.store, &self.scene)
        {
            WorldCupScreenState::HumanJump => self.scene.elements(),
            WorldCupScreenState::DisplayList => {
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
        if self.is_displaying_results() {
            apply_menu_tint(palette, 3, 0);
            return;
        }
        self.scene.apply_palette(palette);
    }
}

impl WorldCupJumpView {
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
