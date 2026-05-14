use crate::data::hill_profile::HillTerrain;
use crate::jump::math::pascal_round;
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

const ANIM_REPLAY_PANEL: u16 = 63; // Pascal Anim[64]
const ANIM_HILL_RECORD_MARKER: u16 = 67; // Pascal Anim[68]
const ANIM_REPLAY_MODE: u16 = 68; // Pascal Anim[69]

pub struct ReplayView {
    resources: ResourcesRef,
    session: RefCell<Option<ReplaySession>>,
    terrain: Result<HillTerrain, String>,
    snow: RefCell<SnowSystem>,
    snow_camera: RefCell<(i32, i32)>,
    snow_advance: Cell<bool>,
    intro_boxes: RefCell<VecDeque<u8>>,
    active_intro_box: RefCell<Option<u8>>,
    shown_intro_boxes: RefCell<[bool; 11]>,
    cursor_blink: Cell<u32>,
    mode: Cell<u8>,
    speed: Cell<u8>,
    place_counter: Cell<u32>,
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
            snow_camera: RefCell::new((0, 0)),
            snow_advance: Cell::new(true),
            intro_boxes: RefCell::new(VecDeque::new()),
            active_intro_box: RefCell::new(None),
            shown_intro_boxes: RefCell::new([false; 11]),
            cursor_blink: Cell::new(0),
            mode: Cell::new(3),
            speed: Cell::new(3),
            place_counter: Cell::new(0),
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

    fn advance_replay_mode(&self, session: &mut ReplaySession) {
        let mode = self.mode.get();
        if mode == 0 {
            return;
        }

        if mode == 3 {
            self.mode.set(0);
            return;
        }

        if mode == 4 {
            self.mode.set(3);
            return;
        }

        let frame = session.frame_index();
        let speed = self.speed.get();
        if matches!(mode, 1 | 2) {
            self.place_counter
                .set(self.place_counter.get().wrapping_add(1));
            if self.place_counter.get() > 999 {
                self.place_counter.set(0);
            }
        }
        let place = self.place_counter.get();
        let advance_by = match speed {
            0 => {
                let flight_start = session.trace().meta.flight_start;
                let flight_stop = session.trace().meta.flight_stop;
                let dist_to_start = (frame as i32 - flight_start as i32).unsigned_abs();
                let dist_to_stop = (frame as i32 - flight_stop as i32).unsigned_abs();
                let effective = if dist_to_start < 20 {
                    1
                } else if dist_to_start < 40 || dist_to_stop < 40 {
                    2
                } else {
                    3
                };
                match effective {
                    1 => i32::from(place.is_multiple_of(4)),
                    2 => (place % 2) as i32,
                    _ => 1,
                }
            }
            1 => i32::from(place.is_multiple_of(4)),
            2 => (place % 2) as i32,
            3 => 1,
            4 => 1 + (place % 2) as i32,
            5 => 2,
            _ => 1,
        };
        match mode {
            1 | 3 | 5 => {
                for _ in 0..advance_by {
                    session.step_forward();
                }
            }
            2 => {
                for _ in 0..advance_by {
                    session.step_back();
                }
            }
            _ => {}
        }
        if session.frame_index() != frame {
            self.snow_advance.set(true);
        }
        if mode == 5 {
            self.mode.set(3);
        }
        if (mode == 1 && session.frame_index() + 1 >= session.trace().frames.len())
            || (mode == 2 && session.frame_index() == 0)
        {
            self.mode.set(3);
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
        let Some(frame) = session.render_frame() else {
            return vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
        };
        let (x, y) = frame.position;
        let (sx, sy) = frame.scroll;
        let replay_frame = frame.replay_frame;

        let mut viewport = terrain
            .viewport_pixels(sx, sy, WIDTH, HEIGHT)
            .as_ref()
            .to_vec();
        if !session.trace().meta.intro {
            let previous = *self.snow_camera.borrow();
            *self.snow_camera.borrow_mut() = (sx, sy);
            let draw = self.snow_advance.replace(false);
            self.snow.borrow_mut().update(
                &mut viewport,
                previous.0 - sx,
                previous.1 - sy,
                i32::from(replay_frame.wind),
                draw,
            );
        }
        let viewport: Rc<[u8]> = viewport.into();
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
            els.push(Element::sprite(
                ANIM_HILL_RECORD_MARKER,
                hr_x - sx,
                hr_y - sy,
            ));
        }
        els.push(Element::sprite(
            u16::from(replay_frame.body_anim),
            x - sx,
            y - sy - 2,
        ));
        els.push(Element::sprite(
            u16::from(replay_frame.ski_anim),
            x - sx,
            y - sy - 1,
        ));

        let wind_pos = WindGaugePosition { x: 10, y: 180 };

        if !session.trace().meta.intro {
            els.push(Element::sprite(ANIM_REPLAY_PANEL, 227, 2));
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
            els.push(Element::sprite(ANIM_REPLAY_MODE, 150, 30));
            els.push(Element::text_color_right(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(340),
                    replay_time(
                        session.frame_index(),
                        session.trace().meta.flight_start,
                        session.trace().meta.flight_stop,
                    )
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
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(342),
                    replay_speed_text(self.speed.get(), &self.resources.langbase)
                ),
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
        presentation::wind_elements(&mut els, wind_pos, i32::from(replay_frame.wind));
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
        } else {
            self.advance_replay_mode(session);
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.active_intro_box.borrow().is_some() {
            return self.dismiss_intro_box();
        }
        match event {
            Event::Keyboard(Key::Escape | Key::Delete) => Some(RouteTarget::Replays),
            Event::Keyboard(Key::Char('+') | Key::Up) => {
                let s = self.speed.get();
                if s < 5 {
                    self.speed.set(s + 1);
                    if self.mode.get() == 0 {
                        self.mode.set(4);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('-') | Key::Down) => {
                let s = self.speed.get();
                if s > 0 {
                    self.speed.set(s - 1);
                    if self.mode.get() == 0 {
                        self.mode.set(4);
                    }
                }
                None
            }
            Event::Keyboard(Key::Right) => {
                self.mode.set(if self.mode.get() == 1 { 3 } else { 1 });
                None
            }
            Event::Keyboard(Key::Left) => {
                self.mode.set(if self.mode.get() == 2 { 3 } else { 2 });
                None
            }
            Event::Keyboard(Key::Char(' ')) if self.mode.get() == 0 => {
                self.mode.set(5);
                None
            }
            Event::Keyboard(Key::Char('p') | Key::Char('P')) => {
                self.mode.set(3);
                None
            }
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(terrain) = &self.terrain {
            terrain.apply_hill_palette(palette);
        }
        muuta_replay(palette, self.mode.get());
    }
}

fn muuta_replay(palette: &mut Palette, mode: u8) {
    let col: u8 = match mode {
        1 => 250,
        2 => 253,
        4 => 251,
        _ => 249,
    };
    for temp in 249..=253 {
        if temp == col {
            palette.set(temp as usize, [10, 63, 20]);
        } else {
            palette.set(temp as usize, [0, 0, 0]);
        }
    }
}

fn format_distance(distance: i32) -> String {
    format!("{:.1}", f64::from(distance) / 10.0)
}

fn replay_time(frame_index: usize, flight_start: usize, flight_stop: usize) -> String {
    if frame_index <= flight_start {
        return "0.00".to_string();
    }
    let temp = if frame_index > flight_stop {
        flight_stop - flight_start
    } else {
        frame_index - flight_start
    };
    let hundredths = pascal_round(temp as f64 * 10.0 / 7.0).max(0);
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
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

fn replay_speed_text(speed: u8, langbase: &crate::parsers::langbase::LangBase) -> String {
    match speed {
        0 => langbase.lstr(343).to_string(),
        1 => "50%".to_string(),
        2 => "75%".to_string(),
        3 => "100%".to_string(),
        4 => "150%".to_string(),
        5 => "200%".to_string(),
        _ => "100%".to_string(),
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
