use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::error::AssetError;
use crate::gfx::palette::{self, BG_LEFT, BLACK, FILL_BORDER, FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::gfx::sprites;
use crate::jump::hud;
use crate::jump::math;
use crate::jump::presentation::{self, WindPosition};
use crate::jump::replay_player::ReplaySession;
use crate::jump::snow::SnowSystem;
use crate::jump::visuals::{self, JumperSpriteSpec};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format;
use crate::text::lang::LangBase;
use crate::views::replay::playback_controls::{PlaybackMode, PlaybackSpeed, ReplayPlayback};
use engine::consts::{HEIGHT, WIDTH};
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::oxide::Blinker;
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

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        let Ok(terrain) = &self.terrain else {
            cx.fill((0, 0, 320, 200), BLACK);
            cx.text((20, 80), FONT_DEFAULT, "Replay hill not found");
            cx.text((20, 95), FONT_HELP, "PRESS ESC");
            return;
        };
        let mut session_ref = self.session.borrow_mut();
        let Some(session) = session_ref.as_mut() else {
            cx.fill((0, 0, 320, 200), BLACK);
            cx.text((20, 80), FONT_DEFAULT, "No replay selected");
            cx.text((20, 95), FONT_HELP, "PRESS ESC");
            return;
        };
        let Some(frame) = session.render_frame() else {
            cx.fill((0, 0, 320, 200), BLACK);
            return;
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
        visuals::push_viewport(cx, &viewport);

        visuals::push_hill_record_marker(cx, session.trace().meta.hill_record_marker, sx, sy);
        visuals::push_jumper_sprites(
            cx,
            JumperSpriteSpec {
                body_anim: u16::from(replay_frame.body_anim),
                ski_anim: u16::from(replay_frame.ski_anim),
                body_x: x - sx,
                body_y: y - sy - 2,
                ski_x: x - sx,
                ski_y: y - sy - 1,
                suit_color: session.trace().meta.suit_color as usize,
                ski_color: session.trace().meta.ski_color as usize,
                has_bib: false,
            },
        );

        let wind_pos = WindPosition { x: 10, y: 180 };

        if !session.trace().meta.intro {
            hud::push_info_panel_frame(cx);
            if session.frame_index() % 30 > 15 {
                cx.text((2, 2), FONT_GOLD, "R");
            }
            let hill_text = self
                .resources
                .hills
                .hill(session.trace().meta.hill_idx)
                .map_or_else(
                    || "?".to_string(),
                    |hill| format!("{} K{}", hill.name, hill.kr),
                );
            cx.right_text((308, 9), FONT_DEFAULT, hill_text);
            cx.right_text((308, 19), FONT_DEFAULT, &session.trace().meta.author);
            cx.sprite_remapped(
                sprites::Sprite::ReplayModeIcon as u16,
                (150, 30),
                palette::replay_speed_recolor(self.playback.mode()),
            );
            cx.right_text(
                (309, 29),
                FONT_GREET,
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(340),
                    replay_time(
                        session.frame_index(),
                        session.trace().meta.flight_start,
                        session.trace().meta.flight_stop,
                    )
                ),
            );
            cx.right_text(
                (309, 39),
                FONT_GREET,
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
            );
            cx.right_text(
                (309, 49),
                FONT_GREET,
                format!(
                    "{} {}",
                    self.resources.langbase.lstr(342),
                    replay_speed_text(self.playback.speed(), &self.resources.langbase)
                ),
            );
            if let Some(gate_text) = replay_gate_text(
                &self.resources.langbase,
                session.trace().meta.start_gate_or_competition,
            ) {
                cx.right_text((309, 59), FONT_GREET, gate_text);
            }
        }
        presentation::wind_elements(cx, wind_pos, i32::from(replay_frame.wind));
        if session.trace().meta.intro {
            self.update_intro_boxes(session.frame_index());
            if let Some(phase) = *self.active_intro_box.borrow() {
                intro_box_elements(
                    cx,
                    &self.resources.langbase,
                    phase,
                    self.cursor_blink.visible(11, 10),
                );
            }
        }
    }
}

impl Screen<RouteTarget> for ReplayView {
    fn update(&mut self) {
        let mut session_ref = self.session.borrow_mut();
        let Some(session) = session_ref.as_mut() else {
            return;
        };

        if session.trace().meta.intro {
            if self.active_intro_box.borrow().is_none() && self.intro_boxes.borrow().is_empty() {
                session.auto_step_forward();
            }
        } else if self.playback.advance(session) {
            self.snow_advance.set(true);
        }
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if self.active_intro_box.borrow().is_some() {
            if let Some(route) = self.dismiss_intro_box() {
                cx.navigate(route);
            } else {
                cx.consume();
            }
            return;
        }
        match event {
            UiEvent::KeyDown(Key::Escape | Key::Delete) => cx.back(),
            UiEvent::KeyDown(Key::Up) | UiEvent::Text('+') => {
                if let Some(s) = self.playback.speed().next_up() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                cx.consume();
            }
            UiEvent::KeyDown(Key::Down) | UiEvent::Text('-') => {
                if let Some(s) = self.playback.speed().next_down() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                cx.consume();
            }
            UiEvent::KeyDown(Key::Right) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Forward {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Forward
                    });
                cx.consume();
            }
            UiEvent::KeyDown(Key::Left) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Rewind {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Rewind
                    });
                cx.consume();
            }
            UiEvent::Text(' ') if self.playback.mode() == PlaybackMode::Pause => {
                self.playback.set_mode(PlaybackMode::OneStep);
                cx.consume();
            }
            UiEvent::Text('p' | 'P') => {
                self.playback.set_mode(PlaybackMode::PlayOnceThenPause);
                cx.consume();
            }
            _ => {}
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn format_distance(distance: i32) -> String {
    format::format_tenths(distance)
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

fn intro_box_elements(
    cx: &mut PaintCx<'_>,
    langbase: &LangBase,
    phase: u8,
    cursor_visible: bool,
) {
    let ix = 30;
    let iy = if phase <= 3 { 140 } else { 30 };
    cx.fill((ix - 7, iy - 7, 269, 40), FILL_BORDER);
    cx.fill((ix - 6, iy - 6, 267, 38), BG_LEFT);
    cx.text((ix, iy), FONT_GOLD, langbase.lstr(360 + phase as usize * 2));
    cx.text((ix, iy + 10), FONT_GOLD, langbase.lstr(361 + phase as usize * 2));
    cx.right_text((ix + 246, iy + 21), FONT_DEFAULT, langbase.lstr(15).to_string());
    cx.fill((ix + 246 + 1 - 2 + 1, iy + 21 - 2, 9, 11), BG_LEFT);
    if cursor_visible {
        cx.fill((ix + 246 + 1, iy + 21 + 6, 5, 1), FONT_DEFAULT);
    }
}
