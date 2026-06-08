use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::error::AssetError;
use crate::gfx::palette::{
    self, BG_LEFT, BLACK, FILL_BORDER, FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP,
    JUMPER_SKI_SOURCE, JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
};
use crate::gfx::sprites;
use crate::jump::math;
use crate::jump::presentation::{self, WindPosition};
use crate::jump::replay_player::ReplaySession;
use crate::jump::snow::SnowSystem;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::lang::LangBase;
use crate::views::replay::playback_controls::{PlaybackMode, PlaybackSpeed, ReplayPlayback};
use engine::consts::{HEIGHT, WIDTH};
use engine::sprite::SpriteColorRecolor;
use engine::ui::{Blinker, Element, Event, ImageRegion, Key, View};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct ReplayView {
    resources: ResourcesRef,
    session: RefCell<Option<ReplaySession>>,
    terrain: Result<HillTerrain, AssetError>,
    snow: RefCell<SnowSystem>,
    snow_camera: RefCell<(i32, i32)>,
    snow_advance: Cell<bool>,
    intro_boxes: RefCell<VecDeque<u8>>,
    active_intro_box: RefCell<Option<u8>>,
    shown_intro_boxes: RefCell<[bool; 11]>,
    cursor_blink: Blinker,
    playback: ReplayPlayback,
}

impl ReplayView {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let trace = store.clone_selected_replay();
        let terrain: Result<HillTerrain, AssetError> = trace.as_ref().map_or_else(
            || Err(AssetError::Custom("Replay hill not found".to_string())),
            |trace| HillTerrain::load(&resources.files, trace.meta.hill_idx),
        );
        let mut snow = SnowSystem::new();
        if let Some(trace) = &trace {
            store.with_jump_rng_wind_mut(|rng, _| snow.set_count(trace.meta.snow_count, rng));
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
            cursor_blink: Blinker::new(),
            playback: ReplayPlayback::new(),
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
            self.cursor_blink.reset();
        }
    }

    fn dismiss_intro_box(&self) -> Option<RouteTarget> {
        let was_last = *self.active_intro_box.borrow() == Some(10);
        let next = self.intro_boxes.borrow_mut().pop_front();
        *self.active_intro_box.borrow_mut() = next;
        self.cursor_blink.reset();
        if was_last && self.active_intro_box.borrow().is_none() {
            Some(RouteTarget::Replays)
        } else {
            None
        }
    }

    fn advance(&self, session: &mut ReplaySession) -> bool {
        self.playback.advance(session)
    }
}

impl View<RouteTarget> for ReplayView {
    fn update(&mut self) {
        let mut session_ref = self.session.borrow_mut();
        let Some(session) = session_ref.as_mut() else {
            return;
        };

        if session.trace().meta.intro {
            if self.active_intro_box.borrow().is_none() && self.intro_boxes.borrow().is_empty() {
                session.auto_step_forward();
            }
        } else if self.advance(session) {
            self.snow_advance.set(true);
        }
    }

    fn elements(&self) -> Vec<Element> {
        let Ok(terrain) = &self.terrain else {
            return vec![
                Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, BLACK),
                Element::text("Replay hill not found", 20, 80, FONT_DEFAULT, false),
                Element::text("PRESS ESC", 20, 95, FONT_HELP, false),
            ];
        };
        let mut session_ref = self.session.borrow_mut();
        let Some(session) = session_ref.as_mut() else {
            return vec![
                Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, BLACK),
                Element::text("No replay selected", 20, 80, FONT_DEFAULT, false),
                Element::text("PRESS ESC", 20, 95, FONT_HELP, false),
            ];
        };
        let Some(frame) = session.render_frame() else {
            return vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, BLACK)];
        };
        let (x, y) = frame.position;
        let (sx, sy) = frame.scroll;
        let replay_frame = frame.replay_frame;

        let (viewport_rgba, viewport_mask) = terrain.viewport_rgba_and_mask(sx, sy, WIDTH, HEIGHT);
        let mut viewport_rgba = viewport_rgba;
        if !session.trace().meta.intro {
            let previous = *self.snow_camera.borrow();
            *self.snow_camera.borrow_mut() = (sx, sy);
            let draw = self.snow_advance.replace(false);
            self.snow.borrow_mut().update(
                &mut viewport_rgba,
                &viewport_mask,
                previous.0 - sx,
                previous.1 - sy,
                i32::from(replay_frame.wind),
                draw,
            );
        }
        let viewport: Rc<[u8]> = viewport_rgba.into();
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
                sprites::Sprite::HillRecordMarker as u16,
                hr_x - sx,
                hr_y - sy,
            ));
        }
        let suit_color = session.trace().meta.suit_color as usize;
        let ski_color = session.trace().meta.ski_color as usize;
        let body_recolor = SpriteColorRecolor::new(vec![
            (
                JUMPER_SUIT_SOURCE_SHADE_1,
                palette::suit_color_shade(suit_color, 1),
            ),
            (
                JUMPER_SUIT_SOURCE_SHADE_3,
                palette::suit_color_shade(suit_color, 3),
            ),
        ]);
        let ski_recolor =
            SpriteColorRecolor::new(vec![(JUMPER_SKI_SOURCE, palette::ski_color(ski_color))]);
        els.push(Element::sprite_remapped(
            u16::from(replay_frame.body_anim),
            x - sx,
            y - sy - 2,
            body_recolor,
        ));
        els.push(Element::sprite_remapped(
            u16::from(replay_frame.ski_anim),
            x - sx,
            y - sy - 1,
            ski_recolor,
        ));

        let wind_pos = WindPosition { x: 10, y: 180 };

        if !session.trace().meta.intro {
            els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
            if session.frame_index() % 30 > 15 {
                els.push(Element::text("R", 2, 2, FONT_GOLD, false));
            }
            let hill_text = self
                .resources
                .hills
                .hill(session.trace().meta.hill_idx)
                .map_or_else(
                    || "?".to_string(),
                    |hill| format!("{} K{}", hill.name, hill.kr),
                );
            els.push(Element::right_text(hill_text, 308, 9, FONT_DEFAULT));
            els.push(Element::text(
                &session.trace().meta.author,
                308,
                19,
                FONT_DEFAULT,
                true,
            ));
            els.push(Element::sprite_remapped(
                sprites::Sprite::ReplayModeIcon as u16,
                150,
                30,
                palette::replay_speed_recolor(self.playback.mode()),
            ));
            els.push(Element::text(
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
                true,
            ));
            els.push(Element::text(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(341),
                    replay_distance(
                        session,
                        self.resources
                            .hills
                            .hill(session.trace().meta.hill_idx)
                            .map_or(1.0, HillInfo::pk),
                    )
                ),
                309,
                39,
                FONT_GREET,
                true,
            ));
            els.push(Element::text(
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(342),
                    replay_speed_text(self.playback.speed(), &self.resources.langbase)
                ),
                309,
                49,
                FONT_GREET,
                true,
            ));
            if let Some(gate_text) = replay_gate_text(
                &self.resources.langbase,
                session.trace().meta.start_gate_or_competition,
            ) {
                els.push(Element::right_text(gate_text, 309, 59, FONT_GREET));
            }
        }
        presentation::wind_elements(&mut els, wind_pos, i32::from(replay_frame.wind));
        if session.trace().meta.intro {
            self.update_intro_boxes(session.frame_index());
            if let Some(phase) = *self.active_intro_box.borrow() {
                intro_box_elements(&mut els, &self.resources.langbase, phase);
                let ix = 30;
                let iy = if phase <= 3 { 140 } else { 30 };
                let blink = self.cursor_blink.visible(11, 10);
                if blink {
                    els.push(Element::fillbox(ix + 247, iy + 27, 5, 1, FONT_DEFAULT));
                }
            }
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.active_intro_box.borrow().is_some() {
            return self.dismiss_intro_box();
        }
        match event {
            Event::Keyboard(Key::Escape | Key::Delete) => Some(RouteTarget::Back),
            Event::Keyboard(Key::Char('+') | Key::Up) => {
                if let Some(s) = self.playback.speed().next_up() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('-') | Key::Down) => {
                if let Some(s) = self.playback.speed().next_down() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                None
            }
            Event::Keyboard(Key::Right) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Forward {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Forward
                    });
                None
            }
            Event::Keyboard(Key::Left) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Rewind {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Rewind
                    });
                None
            }
            Event::Keyboard(Key::Char(' ')) if self.playback.mode() == PlaybackMode::Pause => {
                self.playback.set_mode(PlaybackMode::OneStep);
                None
            }
            Event::Keyboard(Key::Char('p' | 'P')) => {
                self.playback.set_mode(PlaybackMode::PlayOnceThenPause);
                None
            }
            Event::Keyboard(_) => None,
        }
    }
}

fn format_distance(distance: i32) -> String {
    crate::text::format::format_tenths(distance)
}

fn replay_time(frame_index: usize, flight_start: usize, flight_stop: usize) -> String {
    if frame_index <= flight_start {
        return "0.00".to_string();
    }
    let flight_frames = if frame_index > flight_stop {
        flight_stop - flight_start
    } else {
        frame_index - flight_start
    };
    let hundredths = math::round(flight_frames as f64 * 10.0 / 7.0).max(0);
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

fn replay_gate_text(langbase: &LangBase, gate: i32) -> Option<String> {
    match gate {
        1..=5 => Some(langbase.lstr((26 + gate) as usize).to_string()),
        11.. => Some(format!("{} {}", langbase.lstr(58), 100 - gate)),
        _ => None,
    }
}

fn replay_speed_text(speed: PlaybackSpeed, langbase: &LangBase) -> String {
    match speed {
        PlaybackSpeed::Variable => langbase.lstr(343).to_string(),
        PlaybackSpeed::Pct25 => "50%".to_string(),
        PlaybackSpeed::Pct50 => "75%".to_string(),
        PlaybackSpeed::Pct100 => "100%".to_string(),
        PlaybackSpeed::Pct150 => "150%".to_string(),
        PlaybackSpeed::Pct200 => "200%".to_string(),
    }
}

fn intro_box_elements(els: &mut Vec<Element>, langbase: &LangBase, phase: u8) {
    let ix = 30;
    let iy = if phase <= 3 { 140 } else { 30 };
    els.push(Element::fillbox(ix - 7, iy - 7, 269, 40, FILL_BORDER));
    els.push(Element::fillbox(ix - 6, iy - 6, 267, 38, BG_LEFT));
    els.push(Element::text(
        langbase.lstr(360 + phase as usize * 2),
        ix,
        iy,
        FONT_GOLD,
        false,
    ));
    els.push(Element::text(
        langbase.lstr(361 + phase as usize * 2),
        ix,
        iy + 10,
        FONT_GOLD,
        false,
    ));
    els.push(Element::text(
        langbase.lstr(15),
        ix + 246,
        iy + 21,
        FONT_DEFAULT,
        true,
    ));
    els.push(Element::fillbox(ix + 245, iy + 19, 9, 11, BG_LEFT));
}
