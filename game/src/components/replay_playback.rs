use crate::jump::replay_player::ReplaySession;
use std::cell::Cell;

/// Replay playback state machine: mode, speed, frame‑divisor counter.
///
/// Modes (Pascal 1:1):
/// 0 = pause, 1 = forward, 2 = rewind, 3 = play‑once‑then‑pause,
/// 4 = speed‑change, 5 = one‑step.
#[derive(Debug)]
pub struct ReplayPlayback {
    mode: Cell<u8>,
    speed: Cell<u8>,
    place_counter: Cell<u32>,
}

impl ReplayPlayback {
    pub fn new() -> Self {
        Self {
            mode: Cell::new(3),
            speed: Cell::new(3),
            place_counter: Cell::new(0),
        }
    }

    pub fn mode(&self) -> u8 {
        self.mode.get()
    }

    pub fn set_mode(&self, mode: u8) {
        self.mode.set(mode);
    }

    pub fn speed(&self) -> u8 {
        self.speed.get()
    }

    pub fn set_speed(&self, speed: u8) {
        self.speed.set(speed);
    }

    /// Advance one tick. Returns `true` if the session frame position changed
    /// (caller should advance snow rendering).
    pub fn advance(&self, session: &mut ReplaySession) -> bool {
        let mode = self.mode.get();
        if mode == 0 {
            return false;
        }

        if mode == 3 {
            self.mode.set(0);
            return false;
        }

        if mode == 4 {
            self.mode.set(3);
            return false;
        }

        let frame = session.frame_index();
        let speed = self.speed.get();
        if matches!(mode, 1 | 2) {
            let cnt = self.place_counter.get().wrapping_add(1);
            self.place_counter.set(if cnt > 999 { 0 } else { cnt });
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

        let changed = session.frame_index() != frame;

        if mode == 5 {
            self.mode.set(3);
        }
        if (mode == 1 && session.frame_index() + 1 >= session.trace().frames.len())
            || (mode == 2 && session.frame_index() == 0)
        {
            self.mode.set(3);
        }

        changed
    }
}

impl Default for ReplayPlayback {
    fn default() -> Self {
        Self::new()
    }
}
