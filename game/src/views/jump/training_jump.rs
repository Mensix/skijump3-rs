use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::input::{JumpInputAction, JumpInputController};
use crate::views::jump::scene::JumpScene;
use engine::oxide::{ImageRegionDraw, PaintCx, Screen, ScreenEventCx, UiEvent, UpdateCx};
use engine::ui::{Element, Event, Key};
use std::cell::RefCell;

pub struct TrainingJumpView {
    store: StoreRef,
    scene: RefCell<JumpScene>,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = store.practice_hill();
        let participant = JumpParticipant::trainee();
        let start_gate = store.practice_start_gate();
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            StoreRef::clone(&store),
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );

        Self {
            store,
            scene: RefCell::new(scene),
        }
    }

    fn handle_jump_event(&self, event: Event) -> Option<RouteTarget> {
        let action = {
            let scene = self.scene.borrow();
            let mut session = scene.session_mut();
            JumpInputController.handle_event(event, &mut session)
        };
        match action {
            JumpInputAction::None => None,
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            JumpInputAction::SaveReplay => {
                self.scene.borrow().open_save_dialog();
                None
            }
            JumpInputAction::ResetWind => {
                self.store.reset_practice_wind();
                None
            }
            JumpInputAction::ResetJump => {
                let _ = self.scene.borrow().outcome();
                let _ = self.scene.borrow().replay_trace();
                self.scene
                    .borrow()
                    .reset_state(self.store.practice_start_gate());
                None
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                self.store.set_practice_start_gate(start_gate);
                self.store.set_start_gate(start_gate);
                None
            }
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        self.scene.borrow().render(cx);
    }

    fn handle_input(&self, event: Event) -> Option<RouteTarget> {
        let scene = self.scene.borrow();
        if scene.is_save_dialog_active() {
            scene.handle_save_dialog_event(&event);
            None
        } else {
            drop(scene);
            self.handle_jump_event(event)
        }
    }
}

impl Screen<RouteTarget> for TrainingJumpView {
    fn update(&mut self, _cx: &mut UpdateCx) {
        self.scene.borrow_mut().update();
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_event(event) else {
            return;
        };
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
