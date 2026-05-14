use crate::jump::replay::ReplayTrace;

#[derive(Debug, Clone)]
pub struct ReplaySession {
    trace: ReplayTrace,
    absolute_positions: Vec<(i32, i32)>,
    frame: usize,
}

impl ReplaySession {
    #[must_use]
    pub fn new(trace: ReplayTrace) -> Self {
        let mut x = trace.meta.start_x;
        let mut y = trace.meta.start_y;
        let mut absolute_positions = Vec::with_capacity(trace.frames.len());
        for frame in &trace.frames {
            x += i32::from(frame.dx);
            y += i32::from(frame.dy);
            absolute_positions.push((x, y));
        }

        Self {
            trace,
            absolute_positions,
            frame: 0,
        }
    }

    #[must_use]
    pub fn frame_index(&self) -> usize {
        self.frame
    }

    #[must_use]
    pub fn position(&self) -> Option<(i32, i32)> {
        self.absolute_positions.get(self.frame).copied()
    }

    pub fn step_forward(&mut self) {
        if self.frame + 1 < self.trace.frames.len() {
            self.frame += 1;
        }
    }

    pub fn step_back(&mut self) {
        self.frame = self.frame.saturating_sub(1);
    }
}
