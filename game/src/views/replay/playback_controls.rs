use crate::jump::replay_player::{PlaybackMode, ReplaySession};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackSpeed {
    Variable,
    Pct25,
    Pct50,
    Pct100,
    Pct150,
    Pct200,
}

impl PlaybackSpeed {
    pub fn next_up(self) -> Option<Self> {
        match self {
            Self::Variable => Some(Self::Pct25),
            Self::Pct25 => Some(Self::Pct50),
            Self::Pct50 => Some(Self::Pct100),
            Self::Pct100 => Some(Self::Pct150),
            Self::Pct150 => Some(Self::Pct200),
            Self::Pct200 => None,
        }
    }

    pub fn next_down(self) -> Option<Self> {
        match self {
            Self::Variable => None,
            Self::Pct25 => Some(Self::Variable),
            Self::Pct50 => Some(Self::Pct25),
            Self::Pct100 => Some(Self::Pct50),
            Self::Pct150 => Some(Self::Pct100),
            Self::Pct200 => Some(Self::Pct150),
        }
    }
}

#[derive(Debug)]
pub struct ReplayPlayback {
    mode: PlaybackMode,
    speed: PlaybackSpeed,
    place_counter: u32,
    previous_mode: PlaybackMode,
}

impl ReplayPlayback {
    pub const fn new() -> Self {
        Self {
            mode: PlaybackMode::PlayOnceThenPause,
            speed: PlaybackSpeed::Pct100,
            place_counter: 0,
            previous_mode: PlaybackMode::Pause,
        }
    }

    pub const fn mode(&self) -> PlaybackMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: PlaybackMode) {
        self.mode = mode;
    }

    pub const fn speed(&self) -> PlaybackSpeed {
        self.speed
    }

    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
    }

    pub fn advance(
        &mut self,
        session: &mut ReplaySession,
        mut on_step: impl FnMut((i32, i32), (i32, i32), i32),
    ) -> bool {
        let mode = self.mode;
        if mode == PlaybackMode::Pause {
            self.previous_mode = PlaybackMode::Pause;
            return false;
        }

        if mode == PlaybackMode::PlayOnceThenPause {
            self.mode = PlaybackMode::Pause;
            self.previous_mode = PlaybackMode::Pause;
            return false;
        }

        if mode == PlaybackMode::SpeedChange {
            self.mode = PlaybackMode::PlayOnceThenPause;
            self.previous_mode = PlaybackMode::Pause;
            return false;
        }

        let frame = session.frame_index();
        let speed = self.speed;
        if (mode == PlaybackMode::Forward && self.previous_mode != PlaybackMode::Rewind)
            || (mode == PlaybackMode::Rewind && self.previous_mode != PlaybackMode::Forward)
        {
            let cnt = self.place_counter.wrapping_add(1);
            self.place_counter = if cnt > 999 { 0 } else { cnt };
        }
        let place = self.place_counter;
        let advance_by: i32 = match speed {
            PlaybackSpeed::Variable => {
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
            PlaybackSpeed::Pct25 => i32::from(place.is_multiple_of(4)),
            PlaybackSpeed::Pct50 => (place % 2) as i32,
            PlaybackSpeed::Pct100 => 1,
            PlaybackSpeed::Pct150 => 1 + (place % 2) as i32,
            PlaybackSpeed::Pct200 => 2,
        };

        match mode {
            PlaybackMode::Forward | PlaybackMode::PlayOnceThenPause | PlaybackMode::OneStep => {
                for _ in 0..advance_by {
                    let previous_camera = session.camera();
                    let wind = session
                        .current_frame()
                        .map_or(0, |frame| i32::from(frame.wind));
                    if session.step_forward() {
                        on_step(previous_camera, session.camera(), wind);
                    }
                }
            }
            PlaybackMode::Rewind => {
                for _ in 0..advance_by {
                    let previous_camera = session.camera();
                    let wind = session
                        .current_frame()
                        .map_or(0, |frame| i32::from(frame.wind));
                    if session.step_back() {
                        on_step(previous_camera, session.camera(), wind);
                    }
                }
            }
            _ => {}
        }

        let changed = session.frame_index() != frame;

        if mode == PlaybackMode::OneStep {
            self.mode = PlaybackMode::PlayOnceThenPause;
        }
        if (mode == PlaybackMode::Forward
            && session.frame_index() + 1 >= session.trace().frames.len())
            || (mode == PlaybackMode::Rewind && session.frame_index() == 0)
        {
            self.mode = PlaybackMode::PlayOnceThenPause;
        }
        self.previous_mode = mode;

        changed
    }
}

impl Default for ReplayPlayback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::jumper_colors::{SkiIdx, SuitIdx};
    use crate::jump::replay::{ReplayFrame, ReplayMeta, ReplayTrace};

    fn session() -> ReplaySession {
        ReplaySession::new(ReplayTrace {
            meta: ReplayMeta {
                start_x: 0,
                start_y: 0,
                hill_idx: 0,
                snow_count: 1,
                distance: 0,
                flight_start: 0,
                flight_stop: 2,
                hill_record_marker: None,
                hill_filename: String::new(),
                hill_profile: 0,
                suit_color: SuitIdx(0),
                ski_color: SkiIdx(0),
                saved_at: String::new(),
                has_bib: false,
                author: String::new(),
                name: String::new(),
                start_gate_or_competition: 0,
                frame_count: 2,
                checksum: 0,
                valid_checksum: true,
                intro: false,
            },
            frames: vec![
                ReplayFrame {
                    dx: 0,
                    dy: 0,
                    body_anim: 0,
                    ski_anim: 0,
                    wind: 1,
                },
                ReplayFrame {
                    dx: 1,
                    dy: 0,
                    body_anim: 0,
                    ski_anim: 0,
                    wind: 2,
                },
                ReplayFrame {
                    dx: 1,
                    dy: 0,
                    body_anim: 0,
                    ski_anim: 0,
                    wind: 3,
                },
            ],
        })
    }

    #[test]
    fn catch_up_reports_every_crossed_replay_step() {
        let mut playback = ReplayPlayback::new();
        playback.set_mode(PlaybackMode::Forward);
        playback.set_speed(PlaybackSpeed::Pct200);
        let mut session = session();
        let mut winds = Vec::new();

        assert!(playback.advance(&mut session, |_, _, wind| winds.push(wind)));

        assert_eq!(session.frame_index(), 2);
        assert_eq!(winds, [1, 2]);
    }

    #[test]
    fn reversing_direction_does_not_advance_the_cadence_counter() {
        let mut playback = ReplayPlayback::new();
        playback.set_speed(PlaybackSpeed::Pct25);
        playback.set_mode(PlaybackMode::Forward);
        let mut session = session();

        for _ in 0..3 {
            assert!(!playback.advance(&mut session, |_, _, _| {}));
        }
        assert!(playback.advance(&mut session, |_, _, _| {}));
        assert_eq!(session.frame_index(), 1);
        playback.set_mode(PlaybackMode::Rewind);
        assert!(playback.advance(&mut session, |_, _, _| {}));
        assert_eq!(session.frame_index(), 0);
        playback.set_mode(PlaybackMode::Forward);
        assert!(playback.advance(&mut session, |_, _, _| {}));
        assert_eq!(session.frame_index(), 1);
    }
}
