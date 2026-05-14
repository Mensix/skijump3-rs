use crate::data::hill_profile::HillTerrain;
use crate::jump::presentation::{self, WindGaugePosition};
use crate::jump::replay_player::ReplaySession;
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::palette::Palette;
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct ReplayView {
    resources: ResourcesRef,
    session: RefCell<Option<ReplaySession>>,
    terrain: Result<HillTerrain, String>,
    snow: RefCell<SnowSystem>,
    camera: RefCell<(i32, i32)>,
    snow_camera: RefCell<(i32, i32)>,
    snow_frame: RefCell<Option<(i32, i32, i32)>>,
    intro_boxes: RefCell<VecDeque<u8>>,
    active_intro_box: RefCell<Option<u8>>,
    shown_intro_boxes: RefCell<[bool; 11]>,
    cursor_blink: Cell<u32>,
}

impl ReplayView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let trace = store.selected_replay.borrow().clone();
        let terrain = trace
            .as_ref()
            .and_then(|trace| resources.hills.hill(trace.meta.hill_idx))
            .ok_or_else(|| "Replay hill not found".to_string())
            .and_then(HillTerrain::load);
        let mut snow = SnowSystem::new();
        if let Some(trace) = &trace {
            snow.set_count(trace.meta.snow_count, &mut store.rng.borrow_mut());
        }
        Self {
            resources,
            session: RefCell::new(trace.map(ReplaySession::new)),
            terrain,
            snow: RefCell::new(snow),
            camera: RefCell::new((0, 0)),
            snow_camera: RefCell::new((0, 0)),
            snow_frame: RefCell::new(None),
            intro_boxes: RefCell::new(VecDeque::new()),
            active_intro_box: RefCell::new(None),
            shown_intro_boxes: RefCell::new([false; 11]),
            cursor_blink: Cell::new(0),
        }
    }

    fn update_intro_boxes(&self, frame_index: usize) {
        if self.active_intro_box.borrow().is_some() || !self.intro_boxes.borrow().is_empty() {
            return;
        }

        let phases: &[u8] = match frame_index {
            1 => &[0, 1, 2],
            100 => &[3],
            223 => &[4],
            257 => &[5],
            420 => &[6],
            550 => &[7, 8, 9],
            822 => &[10],
            _ => &[],
        };
        if phases.is_empty() {
            return;
        }

        let mut shown = self.shown_intro_boxes.borrow_mut();
        let mut queue = self.intro_boxes.borrow_mut();
        for &phase in phases {
            if !shown[phase as usize] {
                shown[phase as usize] = true;
                queue.push_back(phase);
            }
        }
        if self.active_intro_box.borrow().is_none() {
            *self.active_intro_box.borrow_mut() = queue.pop_front();
            self.cursor_blink.set(0);
        }
    }

    fn dismiss_intro_box(&self) -> Option<RouteTarget> {
        let was_last = *self.active_intro_box.borrow() == Some(10);
        let next = self.intro_boxes.borrow_mut().pop_front();
        *self.active_intro_box.borrow_mut() = next;
        self.cursor_blink.set(0);
        if was_last && self.active_intro_box.borrow().is_none() {
            Some(RouteTarget::Replays)
        } else {
            None
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
        let mut session_ref = self.session.borrow_mut();
        let Some(session) = session_ref.as_mut() else {
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

        let mut sx = { self.camera.borrow().0 };
        let mut sy = { self.camera.borrow().1 };
        if (160..864).contains(&x) {
            sx = x - 160;
        }
        if (100..412).contains(&y) {
            sy = y - 100;
        }
        sx = sx.clamp(0, 704);
        sy = sy.clamp(0, 312);
        *self.snow_frame.borrow_mut() = Some((sx, sy, i32::from(frame.wind)));
        *self.camera.borrow_mut() = (sx, sy);

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

        if let Some((hr_x, hr_y)) = session.trace().meta.hill_record_marker {
            els.push(Element::sprite(67, hr_x - sx, hr_y - sy));
        }
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

        let wind_pos = WindGaugePosition { x: 10, y: 180 };

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
            els.push(Element::sprite(68, 150, 30));
            els.push(Element::text_color_right(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(340),
                    replay_time(session.frame_index(), session.trace().meta.flight_start)
                ),
                309,
                29,
                FONT_GREET,
            ));
            els.push(Element::text_color_right(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(341),
                    replay_distance(
                        session,
                        self.resources
                            .hills
                            .hill(session.trace().meta.hill_idx)
                            .map_or(1.0, |hill| hill.pk()),
                    )
                ),
                309,
                39,
                FONT_GREET,
            ));
            els.push(Element::text_color_right(
                format!("{} 100%", self.resources.langbase.lstr(342)),
                309,
                49,
                FONT_GREET,
            ));
            if let Some(gate_text) = replay_gate_text(
                &self.resources.langbase,
                session.trace().meta.start_gate_or_competition,
            ) {
                els.push(Element::text_color_right(gate_text, 309, 59, FONT_GREET));
            }
        }
        presentation::wind_elements(&mut els, wind_pos, i32::from(frame.wind));
        if session.trace().meta.intro {
            self.update_intro_boxes(session.frame_index());
            if let Some(phase) = *self.active_intro_box.borrow() {
                intro_box_elements(&mut els, &self.resources.langbase, phase);
                let ix = 30;
                let iy = if phase <= 3 { 140 } else { 30 };
                let blink = self.cursor_blink.get();
                self.cursor_blink.set(blink + 1);
                if blink % 21 <= 10 {
                    els.push(Element::fillbox(ix + 247, iy + 27, 5, 1, FONT_DEFAULT));
                }
            } else {
                session.auto_step_forward();
            }
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.active_intro_box.borrow().is_some() {
            return self.dismiss_intro_box();
        }
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

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if self.active_intro_box.borrow().is_some() {
            return;
        }
        if self
            .session
            .borrow()
            .as_ref()
            .is_some_and(|s| s.trace().meta.intro)
        {
            return;
        }
        let Some((sx, sy, wind)) = *self.snow_frame.borrow() else {
            return;
        };
        let previous = *self.snow_camera.borrow();
        *self.snow_camera.borrow_mut() = (sx, sy);
        self.snow
            .borrow_mut()
            .update(framebuffer, previous.0 - sx, previous.1 - sy, wind, true);
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(terrain) = &self.terrain {
            terrain.apply_hill_palette(palette);
        }
    }
}

fn format_distance(distance: i32) -> String {
    format!("{:.1}m", f64::from(distance) / 10.0)
}

fn replay_time(frame_index: usize, flight_start: usize) -> String {
    if frame_index <= flight_start {
        return "0.00".to_string();
    }
    let tenths = (((frame_index - flight_start) as f64 * 10.0 / 7.0) + 0.5).floor() as i32;
    format!("{}.{:02}", tenths / 10, tenths % 10)
}

fn replay_distance(session: &ReplaySession, hill_pk: f64) -> String {
    if session.frame_index() <= session.trace().meta.flight_start {
        return format_distance(0);
    }
    if session.frame_index() > session.trace().meta.flight_stop {
        return format_distance(session.trace().meta.distance);
    }
    let Some((start_x, start_y)) = session.position_at(session.trace().meta.flight_start) else {
        return format_distance(0);
    };
    let Some((x, y)) = session.position() else {
        return format_distance(0);
    };
    let dx = i64::from(x - start_x);
    let dy = i64::from(y - start_y);
    let raw = (((dx * dx + dy * dy) as f64).sqrt() * hill_pk * 0.5).round() as i32 * 5;
    format_distance(raw)
}

fn replay_gate_text(langbase: &crate::parsers::langbase::LangBase, gate: i32) -> Option<String> {
    match gate {
        1..=5 => Some(langbase.lstr((26 + gate) as usize).to_string()),
        11.. => Some(format!("{} {}", langbase.lstr(58), 100 - gate)),
        _ => None,
    }
}

fn intro_box_elements(
    els: &mut Vec<Element>,
    langbase: &crate::parsers::langbase::LangBase,
    phase: u8,
) {
    let ix = 30;
    let iy = if phase <= 3 { 140 } else { 30 };
    els.push(Element::fillbox(ix - 7, iy - 7, 269, 40, 248));
    els.push(Element::fillbox(ix - 6, iy - 6, 267, 38, 243));
    els.push(Element::text_color(
        langbase.lstr(360 + phase as usize * 2),
        ix,
        iy,
        FONT_GOLD,
    ));
    els.push(Element::text_color(
        langbase.lstr(361 + phase as usize * 2),
        ix,
        iy + 10,
        FONT_GOLD,
    ));
    els.push(Element::text_color_right(
        langbase.lstr(15),
        ix + 246,
        iy + 21,
        FONT_DEFAULT,
    ));
    els.push(Element::fillbox(ix + 245, iy + 19, 9, 11, 243));
}
