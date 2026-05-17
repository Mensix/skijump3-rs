use crate::competition::types::CompetitionPhase;
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::controllers::world_cup_flow::{self, WorldCupCommand};
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
    last_event: Cell<usize>,
    result_acknowledged: Cell<bool>,
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
            last_event: Cell::new(0),
            result_acknowledged: Cell::new(false),
            display_page: Cell::new(0),
            render_mode: Cell::new(RenderMode::Jump),
        }
    }

    fn record_finished_human_jump(&self) {
        let outcome = self.scene.outcome();
        if outcome.is_none() || !self.result_acknowledged.get() {
            return;
        }
        let outcome = outcome.unwrap();
        if !self
            .store
            .competition
            .try_with(|c| c.is_human_current())
            .unwrap_or(false)
        {
            return;
        }
        self.store
            .competition
            .try_with_mut(|c| c.record_jump(outcome.score, outcome.distance));
        self.result_acknowledged.set(false);
    }

    fn handle_human_jump(
        &self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    ) {
        let needs_rebuild = self.scene.participant_id() != participant.id
            || self.scene.hill_idx() != hill_idx
            || (self.scene.outcome().is_some() && self.result_acknowledged.get());
        if needs_rebuild {
            self.result_acknowledged.set(false);
            self.scene
                .rebuild_for_competition(hill_idx, 15, participant, phase_label);
        } else {
            self.scene.set_phase_label(phase_label);
        }
    }

    fn results_page(&self) -> Vec<Element> {
        self.store
            .competition
            .try_with(|c| {
                let page_data = competition_results::build_results_page(c, self.display_page.get());
                let mut els = competition_results::render_results_page(&page_data, &self.resources);
                els.extend(competition_results::render_header(c, &self.resources));
                els
            })
            .unwrap_or_else(|| vec![Element::fillbox(0, 0, 320, 200, 0)])
    }

    fn drive_competition(&self) {
        let command = self.store.competition.try_with_mut(|c| {
            let mut simulate_computer = |participant: JumpParticipant,
                                         hill_idx: usize|
             -> crate::jump::types::JumpOutcome {
                self.scene.simulate_hidden(participant, hill_idx)
            };
            world_cup_flow::drive(c, &self.last_event, &mut simulate_computer)
        });

        let Some(command) = command else {
            self.render_mode.set(RenderMode::Done);
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

impl View<RouteTarget> for WorldCupJumpView {
    fn update(&mut self) {
        self.record_finished_human_jump();
        self.drive_competition();
        if self.render_mode.get() == RenderMode::Jump {
            self.scene.update();
        }
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
            return self.handle_result_event(event);
        }

        // Pascal: wait for key after human jump before advancing
        if self.scene.outcome().is_some() && !self.result_acknowledged.get() {
            if matches!(event, Event::Keyboard(Key::Enter | Key::Escape)) {
                self.result_acknowledged.set(true);
                return None;
            }
            return None;
        }

        let action = {
            let mut session = self.scene.session_mut();
            JumpInputController.handle_event(event, &mut session)
        };
        match action {
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            _ => None,
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
        self.store
            .competition
            .try_with(|c| {
                matches!(
                    c.phase(),
                    CompetitionPhase::QualificationResults
                        | CompetitionPhase::Round1Results
                        | CompetitionPhase::Round2Results
                        | CompetitionPhase::WorldCupStandings
                        | CompetitionPhase::SeasonComplete
                ) || matches!(
                    c.phase(),
                    CompetitionPhase::Qualification
                        | CompetitionPhase::Round1
                        | CompetitionPhase::Round2
                ) && c.current_jumper().is_none()
            })
            .unwrap_or(false)
    }

    fn handle_result_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                let page = self.display_page.get();
                let total = self
                    .store
                    .competition
                    .try_with(competition_results::total_pages)
                    .unwrap_or(0);
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
                self.store.competition.try_with_mut(|c| c.advance());
                None
            }
            _ => None,
        }
    }
}
