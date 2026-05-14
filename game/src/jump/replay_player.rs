use crate::jump::replay::{ReplayFrame, ReplayTrace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayRenderFrame {
    pub frame_index: usize,
    pub position: (i32, i32),
    pub scroll: (i32, i32),
    pub replay_frame: ReplayFrame,
}

#[derive(Debug, Clone)]
pub struct ReplaySession {
    trace: ReplayTrace,
    absolute_positions: Vec<(i32, i32)>,
    frame: usize,
    camera: (i32, i32),
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
            camera: (0, 0),
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

    #[must_use]
    pub fn position_at(&self, frame: usize) -> Option<(i32, i32)> {
        self.absolute_positions.get(frame).copied()
    }

    #[must_use]
    pub fn current_frame(&self) -> Option<ReplayFrame> {
        self.trace.frames.get(self.frame).copied()
    }

    pub fn viewport_scroll(&mut self) -> Option<(i32, i32)> {
        let (x, y) = self.position()?;
        let mut sx = self.camera.0;
        let mut sy = self.camera.1;
        if (160..864).contains(&x) {
            sx = x - 160;
        }
        if (100..412).contains(&y) {
            sy = y - 100;
        }
        sx = sx.clamp(0, 704);
        sy = sy.clamp(0, 312);
        self.camera = (sx, sy);
        Some(self.camera)
    }

    pub fn render_frame(&mut self) -> Option<ReplayRenderFrame> {
        Some(ReplayRenderFrame {
            frame_index: self.frame,
            position: self.position()?,
            scroll: self.viewport_scroll()?,
            replay_frame: self.current_frame()?,
        })
    }

    #[must_use]
    pub fn trace(&self) -> &ReplayTrace {
        &self.trace
    }

    pub fn step_forward(&mut self) {
        if self.frame + 1 < self.trace.frames.len() {
            self.frame += 1;
        }
    }

    pub fn auto_step_forward(&mut self) {
        if self.trace.meta.intro {
            self.step_forward();
        }
    }

    pub fn step_back(&mut self) {
        self.frame = self.frame.saturating_sub(1);
    }
}
