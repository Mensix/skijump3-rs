use crate::jump::replay_player::ReplaySession;
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Pause,
    Forward,
    Rewind,
    PlayOnceThenPause,
    SpeedChange,
    OneStep,
}

#[derive(Debug)]
pub struct ReplayPlayback {
    mode: Cell<PlaybackMode>,
    speed: Cell<u8>,
    place_counter: Cell<u32>,
}

impl ReplayPlayback {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: Cell::new(PlaybackMode::PlayOnceThenPause),
            speed: Cell::new(3),
            place_counter: Cell::new(0),
        }
    }

    pub const fn mode(&self) -> PlaybackMode {
        self.mode.get()
    }

    pub fn set_mode(&self, mode: PlaybackMode) {
        self.mode.set(mode);
    }

    pub const fn speed(&self) -> u8 {
        self.speed.get()
    }

    pub fn set_speed(&self, speed: u8) {
        self.speed.set(speed);
    }

    pub fn advance(&self, session: &mut ReplaySession) -> bool {
        let mode = self.mode.get();
        if mode == PlaybackMode::Pause {
            return false;
        }

        if mode == PlaybackMode::PlayOnceThenPause {
            self.mode.set(PlaybackMode::Pause);
            return false;
        }

        if mode == PlaybackMode::SpeedChange {
            self.mode.set(PlaybackMode::PlayOnceThenPause);
            return false;
        }

        let frame = session.frame_index();
        let speed = self.speed.get();
        if matches!(mode, PlaybackMode::Forward | PlaybackMode::Rewind) {
            let cnt = self.place_counter.get().wrapping_add(1);
            self.place_counter.set(if cnt > 999 { 0 } else { cnt });
        }
        let place = self.place_counter.get();
        let advance_by: i32 = match speed {
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
            PlaybackMode::Forward | PlaybackMode::PlayOnceThenPause | PlaybackMode::OneStep => {
                for _ in 0..advance_by {
                    session.step_forward();
                }
            }
            PlaybackMode::Rewind => {
                for _ in 0..advance_by {
                    session.step_back();
                }
            }
            _ => {}
        }

        let changed = session.frame_index() != frame;

        if mode == PlaybackMode::OneStep {
            self.mode.set(PlaybackMode::PlayOnceThenPause);
        }
        if (mode == PlaybackMode::Forward
            && session.frame_index() + 1 >= session.trace().frames.len())
            || (mode == PlaybackMode::Rewind && session.frame_index() == 0)
        {
            self.mode.set(PlaybackMode::PlayOnceThenPause);
        }

        changed
    }
}

impl Default for ReplayPlayback {
    fn default() -> Self {
        Self::new()
    }
}
