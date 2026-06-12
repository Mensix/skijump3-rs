use super::setup::{SetupAction, TeamCupSetup};
use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind, TeamCupRuntime};
use crate::components::screen;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::team_cup::results as team_cup_results;
use crate::views::jump::competition::ui_state::RenderMode;
use engine::oxide::{ImageRegionDraw, PaintCx, Screen, ScreenEventCx, UiEvent, UpdateCx};
use engine::ui::{Blinker, Element, Event, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewPhase {
    Setup,
    Jumping,
    Done,
}

pub struct TeamCupJumpView {
    controller: CompetitionJumpController<TeamCupRuntime>,
    phase: ViewPhase,
    blinker: Blinker,
    setup: TeamCupSetup,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let setup = TeamCupSetup::new(&store);
        Self {
            controller: CompetitionJumpController::new(resources, store, None),
            phase: ViewPhase::Setup,
            blinker: Blinker::new(),
            setup,
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn drive_until_visible(&mut self) {
        if let Some(cmd) = self.controller.drive() {
            self.apply_command(cmd);
        } else if self.controller.render_mode() != RenderMode::Error {
            self.controller.enter_error("No competition running");
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<TeamCupJumpContext, TeamCupResultsKind>,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => {
                let phase_label = if context.round_idx == 0 {
                    self.controller.resources().langbase.lstr(54).to_string()
                } else {
                    self.controller.resources().langbase.lstr(55).to_string()
                };
                self.controller.prepare_human_jump(
                    participant,
                    hill_idx,
                    phase_label,
                    Some(context.team_name.clone()),
                );
            }
            CompetitionFlowCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.phase = ViewPhase::Done;
                self.controller.enter_done();
            }
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        if self.phase == ViewPhase::Setup {
            self.setup.paint(
                cx,
                self.controller.resources(),
                self.controller.store(),
                self.cursor_visible,
            );
            return;
        }
        if self.phase == ViewPhase::Done {
            return;
        }

        match self.controller.render_mode() {
            RenderMode::Jump => {
                let items = self.controller.render_jump_elements();
                draw_items(cx, &items);
            }
            RenderMode::Results => {
                let els = team_cup_results::render(
                    self.controller.resources(),
                    self.controller.store(),
                    self.results_kind,
                );
                draw_items(cx, &els);
            }
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.controller.render_mode() == RenderMode::Error {
                    self.controller.ui_state().error_message()
                } else {
                    String::new()
                };
                let items = screen::message_screen(&msg, self.controller.resources().langbase.lstr(15));
                draw_items(cx, &items);
            }
        }
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        let legacy = input_event(event);
        let Some(legacy) = legacy else {
            return None;
        };

        if let Some(route) = route_error_back(self.controller.ui_state(), legacy) {
            return Some(route);
        }
        if self.controller.render_mode() == RenderMode::Error {
            return None;
        }

        if self.phase == ViewPhase::Done {
            return Some(RouteTarget::Back);
        }

        if self.phase == ViewPhase::Setup {
            if self
                .setup
                .handle_event(self.controller.resources(), self.controller.store(), event)
                == SetupAction::StartJumping
            {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible();
            }
            return None;
        }

        // Let the shared input controller process events first
        if self.phase == ViewPhase::Jumping {
            match self
                .controller
                .handle_jump_scene_event(legacy, false, false, true)
            {
                JumpInputResult::Route(route) => return Some(route),
                JumpInputResult::Consumed => return None,
                JumpInputResult::None => {}
            }
        }

        if self.controller.render_mode() == RenderMode::Results {
            if matches!(legacy, Event::Keyboard(_)) {
                if let Some(cmd) = self
                    .controller
                    .dismiss_results_and_advance(self.results_kind)
                {
                    self.apply_command(cmd);
                }
            }
            return None;
        }

        None
    }
}

impl Screen<RouteTarget> for TeamCupJumpView {
    fn update(&mut self, _cx: &mut UpdateCx) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if self.phase == ViewPhase::Setup {
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        self.controller.record_acknowledged_human_jump();

        // Drive competition only after human jump outcome is recorded,
        // not every frame during the jump (avoids recreating the scene).
        if self.controller.ui_state().is_outcome_recorded()
            && self.controller.render_mode() != RenderMode::Results
        {
            if let Some(cmd) = self.controller.drive() {
                self.apply_command(cmd);
            }
        }

        self.controller.update_scene();
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn input_event(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(c) => Some(Event::Keyboard(Key::Char(c))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

fn draw_items(cx: &mut PaintCx<'_>, items: &[Element]) {
    for item in items {
        draw_item(cx, item);
    }
}

fn draw_item(cx: &mut PaintCx<'_>, item: &Element) {
    match item {
        Element::Image(pixels, w, h) => cx.image(pixels.clone(), *w, *h),
        Element::ImageRegion(region) => cx.image_region(ImageRegionDraw {
            pixels: region.pixels.clone(),
            src_w: region.src_w,
            src_h: region.src_h,
            src_x: region.src_x,
            src_y: region.src_y,
            dst_x: region.dst_x,
            dst_y: region.dst_y,
            w: region.w,
            h: region.h,
        }),
        Element::Text {
            text,
            x,
            y,
            color,
            right,
            center,
        } => {
            if *center {
                cx.center_text((*x, *y), *color, text);
            } else if *right {
                cx.right_text((*x, *y), *color, text);
            } else {
                cx.text((*x, *y), *color, text);
            }
        }
        Element::Sprite(idx, x, y) => cx.sprite(*idx, (*x, *y)),
        Element::Fillbox { x, y, w, h, color } => cx.fill((*x, *y, *w, *h), *color),
        Element::FillArea { thing } => cx.dither_fill(*thing),
        Element::Box { x, y, w, h, color } => cx.stroke((*x, *y, *w, *h), *color),
        Element::SpriteRemapped(idx, x, y, recolor) => {
            cx.sprite_remapped(*idx, (*x, *y), recolor.clone());
        }
        Element::Container(children) => draw_items(cx, children),
    }
}
