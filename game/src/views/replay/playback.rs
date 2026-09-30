use crate::data::hill_profile::HillTerrain;
use crate::gfx::materials;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::jump::hud;
use crate::jump::math;
use crate::jump::presentation::{self, WindPosition};
use crate::jump::replay::ReplayTrace;
use crate::jump::replay_player::{PlaybackMode, ReplaySession};
use crate::jump::snow::SnowSystem;
use crate::jump::visuals::{self, JumperSpriteSpec};
use crate::rng::Random;
use crate::route::{ReplayReturn, RouteTarget};
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::text::format;
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;
use crate::ui::{Blinker, Key, ScreenEventCx, UiEvent};
use crate::views::replay::playback_controls::{PlaybackSpeed, ReplayPlayback};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct ReplayView {
    resources: ResourcesRef,
    session: Option<ReplaySession>,
    terrain: Option<Rc<HillTerrain>>,
    snow: SnowSystem,
    previous_camera: (i32, i32),
    intro_boxes: VecDeque<u8>,
    active_intro_box: Option<u8>,
    shown_intro_boxes: [bool; 11],
    cursor_blink: Blinker,
    playback: ReplayPlayback,
    return_to: ReplayReturn,
}

impl ReplayView {
    pub fn new(
        resources: ResourcesRef,
        trace: ReplayTrace,
        return_to: ReplayReturn,
        low_graphics_detail: bool,
    ) -> Self {
        let mut trace = trace;
        let terrain = resources
            .hills
            .replay_hill_index(
                trace.meta.hill_idx,
                &trace.meta.hill_filename,
                trace.meta.hill_profile,
            )
            .inspect(|hill_idx| {
                trace.meta.hill_idx = *hill_idx;
            })
            .map(|hill_idx| resources.terrain(hill_idx));
        let mut snow = SnowSystem::new();
        snow.set_count(
            replay_snow_count(trace.meta.snow_count, low_graphics_detail),
            &mut Random::default(),
        );
        let mut view = Self {
            resources,
            session: Some(ReplaySession::new(trace)),
            terrain,
            snow,
            previous_camera: (0, 0),
            intro_boxes: VecDeque::new(),
            active_intro_box: None,
            shown_intro_boxes: [false; 11],
            cursor_blink: Blinker::new(),
            playback: ReplayPlayback::new(),
            return_to,
        };
        view.advance_visuals();
        view
    }

    fn update_intro_boxes(&mut self, frame_index: usize) {
        if self.active_intro_box.is_some() || !self.intro_boxes.is_empty() {
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

        for &phase in phases {
            if !self.shown_intro_boxes[phase as usize] {
                self.shown_intro_boxes[phase as usize] = true;
                self.intro_boxes.push_back(phase);
            }
        }
        if self.active_intro_box.is_none() {
            self.active_intro_box = self.intro_boxes.pop_front();
            self.cursor_blink.reset();
        }
    }

    fn dismiss_intro_box(&mut self) -> Option<ReplayReturn> {
        let was_last = self.active_intro_box == Some(10);
        self.active_intro_box = self.intro_boxes.pop_front();
        self.cursor_blink.reset();
        if was_last && self.active_intro_box.is_none() {
            Some(self.return_to)
        } else {
            None
        }
    }

    fn advance_visuals(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        let camera = session.camera();
        self.snow.advance(
            self.previous_camera.0 - camera.0,
            self.previous_camera.1 - camera.1,
            session
                .current_frame()
                .map_or(0, |frame| i32::from(frame.wind)),
        );
        self.previous_camera = camera;
    }

    fn paint_content(&mut self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let Some(terrain) = &self.terrain else {
            cx.fill((0, 0, 320, 200), BLACK);
            cx.text((20, 80), FONT_BODY, "Replay hill not found");
            cx.text((20, 95), FONT_GRAY, "PRESS ESC");
            return;
        };
        let Some(session) = self.session.as_ref() else {
            cx.fill((0, 0, 320, 200), BLACK);
            cx.text((20, 80), FONT_BODY, "No replay selected");
            cx.text((20, 95), FONT_GRAY, "PRESS ESC");
            return;
        };
        let Some(frame) = session.render_frame() else {
            cx.fill((0, 0, 320, 200), BLACK);
            return;
        };
        let (x, y) = frame.position;
        let (sx, sy) = frame.scroll;
        let replay_frame = frame.replay_frame;

        let snow_pixels = if session.trace().meta.intro {
            engine::oxide::PointBatches::empty()
        } else {
            self.snow.pixel_draws()
        };
        visuals::push_hill_layers(
            cx,
            &terrain.back_layer(),
            &terrain.front_layer(),
            snow_pixels,
            sx,
            sy,
            None,
        );

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
                suit_color: session.trace().meta.suit_color.rgb(),
                ski_color: session.trace().meta.ski_color.rgb(),
                has_bib: session.trace().meta.has_bib,
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
            cx.right_text((308, 9), FONT_BODY, &hill_text);
            cx.right_text((308, 19), FONT_BODY, &session.trace().meta.author);
            cx.sprite_with_material(
                sprites::Sprite::ReplayModeIcon as u16,
                (150, 30),
                materials::replay_speed_material(self.playback.mode()),
            );
            cx.right_text(
                (309, 29),
                FONT_TEAL,
                &format!(
                    "{} {}",
                    lang.tr(340),
                    replay_time(
                        session.frame_index(),
                        session.trace().meta.flight_start,
                        session.trace().meta.flight_stop,
                    )
                ),
            );
            cx.right_text(
                (309, 39),
                FONT_TEAL,
                &format!(
                    "{} {}",
                    lang.tr(341),
                    replay_distance(
                        session,
                        self.resources
                            .hills
                            .hill(session.trace().meta.hill_idx)
                            .map_or(1.0, |hill| hill.pk()),
                    )
                ),
            );
            cx.right_text(
                (309, 49),
                FONT_TEAL,
                &format!(
                    "{} {}",
                    lang.tr(342),
                    replay_speed_text(self.playback.speed(), &self.resources.langbase)
                ),
            );
            if let Some(gate_text) = replay_gate_text(
                &self.resources.langbase,
                session.trace().meta.start_gate_or_competition,
            ) {
                cx.right_text((309, 59), FONT_TEAL, &gate_text);
            }
        }
        presentation::wind_elements(cx, wind_pos, i32::from(replay_frame.wind));
        if session.trace().meta.intro {
            if let Some(phase) = self.active_intro_box {
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

fn replay_snow_count(recorded_count: u16, low_graphics_detail: bool) -> u16 {
    if low_graphics_detail {
        0
    } else {
        recorded_count
    }
}

impl GameScreen for ReplayView {
    fn update(&mut self, _: &mut GameCx<'_>) {
        let Some(is_intro) = self
            .session
            .as_ref()
            .map(|session| session.trace().meta.intro)
        else {
            return;
        };

        if is_intro {
            if self.active_intro_box.is_none() && self.intro_boxes.is_empty() {
                let frame_index = {
                    let Some(session) = self.session.as_mut() else {
                        return;
                    };
                    session.auto_step_forward();
                    session.frame_index()
                };
                self.update_intro_boxes(frame_index);
                self.advance_visuals();
            }
        } else {
            let snow = &mut self.snow;
            let Some(session) = self.session.as_mut() else {
                return;
            };
            self.playback.advance(session, |previous, camera, wind| {
                snow.advance(previous.0 - camera.0, previous.1 - camera.1, wind);
            });
        }
    }

    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if self.active_intro_box.is_some() {
            if let Some(return_to) = self.dismiss_intro_box() {
                return_to.navigate(nav);
            } else {
                nav.consume();
            }
            return;
        }
        match event {
            UiEvent::KeyDown(Key::Escape | Key::F10 | Key::Delete) => self.return_to.navigate(nav),
            UiEvent::KeyDown(Key::Up) | UiEvent::Text('+') => {
                if let Some(s) = self.playback.speed().next_up() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                nav.consume();
            }
            UiEvent::KeyDown(Key::Down) | UiEvent::Text('-') => {
                if let Some(s) = self.playback.speed().next_down() {
                    self.playback.set_speed(s);
                    if self.playback.mode() == PlaybackMode::Pause {
                        self.playback.set_mode(PlaybackMode::SpeedChange);
                    }
                }
                nav.consume();
            }
            UiEvent::KeyDown(Key::Right) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Forward {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Forward
                    });
                nav.consume();
            }
            UiEvent::KeyDown(Key::Left) => {
                self.playback
                    .set_mode(if self.playback.mode() == PlaybackMode::Rewind {
                        PlaybackMode::PlayOnceThenPause
                    } else {
                        PlaybackMode::Rewind
                    });
                nav.consume();
            }
            UiEvent::Text(' ') if self.playback.mode() == PlaybackMode::Pause => {
                self.playback.set_mode(PlaybackMode::OneStep);
                nav.consume();
            }
            UiEvent::Text('p' | 'P') => {
                self.playback.set_mode(PlaybackMode::PlayOnceThenPause);
                nav.consume();
            }
            _ => {}
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint);
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
    let Some((x, y)) = session.position_at(session.frame_index()) else {
        return format_distance(0);
    };
    let dx = i64::from(x - start_x);
    let dy = i64::from(y - start_y);
    let raw = (((dx * dx + dy * dy) as f64).sqrt() * hill_pk * 0.5).round() as i32 * 5;
    format_distance(raw)
}

fn replay_gate_text(lang: &LangBase, gate: i32) -> Option<String> {
    match gate {
        1..=5 => Some(lang.tr((26 + gate) as usize).to_string()),
        11.. => Some(format!("{} {}", lang.tr(58), 100 - gate)),
        _ => None,
    }
}

fn replay_speed_text(speed: PlaybackSpeed, lang: &LangBase) -> String {
    match speed {
        PlaybackSpeed::Variable => lang.tr(343).to_string(),
        PlaybackSpeed::Pct25 => "50%".to_string(),
        PlaybackSpeed::Pct50 => "75%".to_string(),
        PlaybackSpeed::Pct100 => "100%".to_string(),
        PlaybackSpeed::Pct150 => "150%".to_string(),
        PlaybackSpeed::Pct200 => "200%".to_string(),
    }
}

fn intro_box_elements(cx: &mut dyn UiCanvas, lang: &LangBase, phase: u8, cursor_visible: bool) {
    let ix = 30;
    let iy = if phase <= 3 { 140 } else { 30 };
    cx.fill((ix - 7, iy - 7, 269, 40), FILL_PURPLE);
    cx.fill((ix - 6, iy - 6, 267, 38), BG_PURPLE);
    cx.text((ix, iy), FONT_GOLD, lang.tr(360 + phase as usize * 2));
    cx.text((ix, iy + 10), FONT_GOLD, lang.tr(361 + phase as usize * 2));
    cx.right_text((ix + 246, iy + 21), FONT_BODY, lang.tr(15));
    cx.fill((ix + 246 + 1 - 2 + 1, iy + 21 - 2, 9, 11), BG_PURPLE);
    if cursor_visible {
        cx.fill((ix + 246 + 1, iy + 21 + 6, 5, 1), FONT_BODY);
    }
}

#[cfg(test)]
mod tests {
    use super::replay_snow_count;

    #[test]
    fn low_graphics_detail_suppresses_replay_snow() {
        assert_eq!(replay_snow_count(1175, true), 0);
        assert_eq!(replay_snow_count(1175, false), 1175);
    }
}
