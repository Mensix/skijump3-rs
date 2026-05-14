use crate::data::hill_profile::HillTerrain;
use crate::jump::replay_player::ReplaySession;
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ReplayView {
    resources: ResourcesRef,
    session: RefCell<Option<ReplaySession>>,
    terrain: Result<HillTerrain, String>,
}

impl ReplayView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let trace = store.selected_replay.borrow().clone();
        let terrain = trace
            .as_ref()
            .and_then(|trace| resources.hills.hill(trace.meta.hill_idx))
            .ok_or_else(|| "Replay hill not found".to_string())
            .and_then(HillTerrain::load);
        Self {
            resources,
            session: RefCell::new(trace.map(ReplaySession::new)),
            terrain,
        }
    }
}

impl View<RouteTarget> for ReplayView {
    fn elements(&self) -> Vec<Element> {
        let Ok(terrain) = &self.terrain else {
            return vec![
                Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0),
                Element::text_color("Replay hill not found", 20, 80, FONT_DEFAULT),
                Element::text_color("PRESS ESC", 20, 95, FONT_HELP),
            ];
        };
        let session_ref = self.session.borrow();
        let Some(session) = session_ref.as_ref() else {
            return vec![
                Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0),
                Element::text_color("No replay selected", 20, 80, FONT_DEFAULT),
                Element::text_color("PRESS ESC", 20, 95, FONT_HELP),
            ];
        };
        let Some((x, y)) = session.position() else {
            return vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
        };
        let Some(frame) = session.current_frame() else {
            return vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
        };

        let mut sx = if (160..864).contains(&x) { x - 160 } else { 0 };
        let mut sy = if (100..412).contains(&y) { y - 100 } else { 0 };
        sx = sx.clamp(0, 704);
        sy = sy.clamp(0, 312);

        let viewport = terrain.viewport_pixels(sx, sy, WIDTH, HEIGHT);
        let mut els = vec![Element::image_region(ImageRegion {
            pixels: Rc::clone(&viewport),
            src_w: WIDTH,
            src_h: HEIGHT,
            src_x: 0,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            w: WIDTH,
            h: HEIGHT,
        })];
        els.push(Element::sprite(
            u16::from(frame.body_anim),
            x - sx,
            y - sy - 2,
        ));
        els.push(Element::sprite(
            u16::from(frame.ski_anim),
            x - sx,
            y - sy - 1,
        ));

        if !session.trace().meta.intro {
            els.push(Element::sprite(63, 227, 2));
            if session.frame_index() % 30 > 15 {
                els.push(Element::text_color("R", 2, 2, FONT_GOLD));
            }
            let hill_text = self
                .resources
                .hills
                .hill(session.trace().meta.hill_idx)
                .map(|hill| format!("{} K{}", hill.name, hill.kr))
                .unwrap_or_else(|| "?".to_string());
            els.push(Element::text_color_right(hill_text, 308, 9, FONT_DEFAULT));
            els.push(Element::text_color_right(
                &session.trace().meta.author,
                308,
                19,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color_right(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(341),
                    format_distance(session.trace().meta.distance)
                ),
                309,
                39,
                FONT_DEFAULT,
            ));
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Replays),
            Event::Keyboard(Key::Right | Key::Char(' ')) => {
                if let Some(session) = self.session.get_mut() {
                    session.step_forward();
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                if let Some(session) = self.session.get_mut() {
                    session.step_back();
                }
                None
            }
            _ => None,
        }
    }
}

fn format_distance(distance: i32) -> String {
    format!("{:.1}m", f64::from(distance) / 10.0)
}
