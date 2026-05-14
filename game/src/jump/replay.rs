#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayFrame {
    pub dx: i8,
    pub dy: i8,
    pub body_anim: u8,
    pub ski_anim: u8,
    pub wind: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMeta {
    pub start_x: i32,
    pub start_y: i32,
    pub hill_idx: usize,
    pub snow_count: u16,
    pub distance: i32,
    pub flight_start: usize,
    pub flight_stop: usize,
    pub hill_record_marker: Option<(i32, i32)>,
    pub author: String,
    pub name: String,
    pub start_gate_or_competition: i32,
    pub frame_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayTrace {
    pub meta: ReplayMeta,
    pub frames: Vec<ReplayFrame>,
}

#[derive(Debug, Clone, Default)]
pub struct ReplayRecorder {
    meta: Option<ReplayMeta>,
    frames: Vec<ReplayFrame>,
}

impl ReplayRecorder {
    pub fn start(&mut self, meta: ReplayMeta) {
        self.meta = Some(meta);
        self.frames.clear();
    }

    pub fn record_frame(
        &mut self,
        previous_pos: (i32, i32),
        current_pos: (i32, i32),
        body_anim: u16,
        ski_anim: u16,
        wind: i32,
    ) {
        if self.meta.is_none() || self.frames.len() > 1000 {
            return;
        }

        self.frames.push(ReplayFrame {
            dx: (current_pos.0 - previous_pos.0).clamp(-128, 127) as i8,
            dy: (current_pos.1 - previous_pos.1).clamp(-128, 127) as i8,
            body_anim: body_anim.min(u16::from(u8::MAX)) as u8,
            ski_anim: ski_anim.min(u16::from(u8::MAX)) as u8,
            wind: wind.clamp(-128, 127) as i8,
        });
    }

    pub fn mark_flight_start(&mut self) {
        if let Some(meta) = &mut self.meta {
            if meta.flight_start == 0 {
                meta.flight_start = self.frames.len();
            }
        }
    }

    pub fn mark_flight_stop(&mut self) {
        if let Some(meta) = &mut self.meta {
            meta.flight_stop = self.frames.len();
        }
    }

    pub fn set_distance(&mut self, distance: i32) {
        if let Some(meta) = &mut self.meta {
            meta.distance = distance;
        }
    }

    pub fn finish(&self) -> Option<ReplayTrace> {
        let mut meta = self.meta.clone()?;
        meta.frame_count = self.frames.len();
        Some(ReplayTrace {
            meta,
            frames: self.frames.clone(),
        })
    }
}
