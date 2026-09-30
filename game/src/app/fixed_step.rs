use std::time::{Duration, Instant};

const NANOS_PER_SECOND: u128 = 1_000_000_000;

pub(super) struct FixedStepClock {
    ticks_per_second: u32,
    max_catch_up_ticks: usize,
    accumulated_tick_nanos: u128,
    last: Instant,
}

impl FixedStepClock {
    pub(super) fn new(ticks_per_second: u32, max_catch_up_ticks: usize) -> Self {
        assert!(ticks_per_second > 0);
        assert!(max_catch_up_ticks > 0);
        Self {
            ticks_per_second,
            max_catch_up_ticks,
            accumulated_tick_nanos: 0,
            last: Instant::now(),
        }
    }

    pub(super) fn tick(&mut self) -> usize {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        self.last = now;
        self.advance_by(elapsed)
    }

    fn advance_by(&mut self, elapsed: Duration) -> usize {
        self.accumulated_tick_nanos = self.accumulated_tick_nanos.saturating_add(
            elapsed
                .as_nanos()
                .saturating_mul(u128::from(self.ticks_per_second)),
        );

        let pending = self.accumulated_tick_nanos / NANOS_PER_SECOND;
        let ticks = pending.min(self.max_catch_up_ticks as u128) as usize;
        if pending > self.max_catch_up_ticks as u128 {
            // Drop stale whole ticks after a stall, but keep the clock's fractional phase.
            self.accumulated_tick_nanos %= NANOS_PER_SECOND;
        } else {
            self.accumulated_tick_nanos -= ticks as u128 * NANOS_PER_SECOND;
        }
        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_exact_rate_across_small_elapsed_intervals() {
        let mut clock = FixedStepClock::new(70, 5);
        let ticks: usize = (0..100)
            .map(|_| clock.advance_by(Duration::from_millis(10)))
            .sum();

        assert_eq!(ticks, 70);
    }

    #[test]
    fn preserves_fractional_elapsed_time() {
        let mut clock = FixedStepClock::new(70, 5);

        assert_eq!(clock.advance_by(Duration::from_millis(10)), 0);
        assert_eq!(clock.advance_by(Duration::from_millis(5)), 1);
        assert_eq!(clock.advance_by(Duration::from_millis(13)), 0);
        assert_eq!(clock.advance_by(Duration::from_millis(1)), 1);
    }

    #[test]
    fn caps_catch_up_and_discards_stale_backlog() {
        let mut clock = FixedStepClock::new(70, 5);

        assert_eq!(clock.advance_by(Duration::from_secs(1)), 5);
        assert_eq!(clock.advance_by(Duration::ZERO), 0);
        assert_eq!(clock.advance_by(Duration::from_millis(15)), 1);
    }
}
