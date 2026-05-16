use crate::utils::pascal_decode;
use std::fmt::Write;

const REPLAY_FRAME_CAPACITY: usize = 1001;
const REPLAY_CHECK_XOR: i32 = 3_675_433;
const NUM_WC_HILLS: usize = 20;

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
    pub hill_filename: String,
    pub hill_profile: i32,
    pub suit_color: u8,
    pub ski_color: u8,
    pub saved_at: String,
    pub has_bib: bool,
    pub author: String,
    pub name: String,
    pub start_gate_or_competition: i32,
    pub frame_count: usize,
    pub checksum: i32,
    pub valid_checksum: bool,
    pub intro: bool,
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
    stopped: bool,
}

impl ReplayRecorder {
    pub fn start(&mut self, meta: ReplayMeta) {
        self.meta = Some(meta);
        self.frames.clear();
        self.stopped = false;
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn record_frame(
        &mut self,
        previous_pos: (i32, i32),
        current_pos: (i32, i32),
        body_anim: u16,
        ski_anim: u16,
        wind: i32,
    ) {
        if self.stopped || self.meta.is_none() || self.frames.len() >= REPLAY_FRAME_CAPACITY {
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
        meta.frame_count = self.frames.len().saturating_sub(1);
        Some(ReplayTrace {
            meta,
            frames: self.frames.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    MissingLine(&'static str),
    InvalidNumber { field: &'static str, value: String },
    MissingReplayData,
    MissingFrameData { expected: usize, actual: usize },
}

impl ReplayTrace {
    pub fn from_sjr_bytes(data: &[u8], intro: bool) -> Result<Self, ReplayError> {
        let mut parser = ReplayParser::new(data);
        let start_x = parser.i32_line("start_x")?;
        let start_y = parser.i32_line("start_y")?;
        let max_turns = parser.usize_line("max_turns")?;
        let hill_idx = parser.usize_line("hill_idx")?;
        let hill_filename = parser.string_line("hill_filename")?;
        let hill_filename_raw = parser.previous_raw_line();
        let hill_profile = parser.i32_line("hill_profile")?;
        let snow_count = parser.u16_line("snow_count")?;
        let distance = parser.i32_line("distance")?;
        let flight_start = parser.usize_line("flight_start")?;
        let flight_stop = parser.usize_line("flight_stop")?;
        let hr_x = parser.i32_line("hill_record_x")?;
        let hr_y = parser.i32_line("hill_record_y")?;
        let suit_color = parser.u8_line("suit_color")?;
        let ski_color = parser.u8_line("ski_color")?;
        let author = {
            parser.line("author")?;
            pascal_decode(parser.previous_raw_line())
        };
        let author_raw = parser.previous_raw_line();
        let name = {
            parser.line("name")?;
            pascal_decode(parser.previous_raw_line())
        };
        let saved_at = parser.string_line("saved_at")?;
        let has_bib = parser.i32_line("has_bib")? != 0;
        let start_gate_or_competition = parser.i32_line("start_gate_or_competition")?;
        let checksum = parser.i32_line("checksum")?;
        let _reserved = parser.string_line("reserved")?;
        let replay_data = parser.replay_data()?;

        let frames = decode_frames(replay_data, max_turns)?;
        let expected_checksum = replay_checksum(ReplayChecksumInput {
            start_x,
            start_y,
            max_turns,
            hill_idx,
            hill_filename: hill_filename_raw,
            hill_profile,
            distance,
            flight_start,
            flight_stop,
            hill_record_x: hr_x,
            hill_record_y: hr_y,
            author: author_raw,
            start_gate_or_competition,
        });

        Ok(Self {
            meta: ReplayMeta {
                start_x,
                start_y,
                hill_idx,
                snow_count,
                distance,
                flight_start,
                flight_stop,
                hill_record_marker: (hr_x > 0).then_some((hr_x, hr_y)),
                hill_filename,
                hill_profile,
                suit_color,
                ski_color,
                saved_at,
                has_bib,
                author,
                name,
                start_gate_or_competition,
                frame_count: max_turns,
                checksum,
                valid_checksum: checksum == expected_checksum,
                intro,
            },
            frames,
        })
    }

    pub fn to_sjr_bytes(&self) -> Vec<u8> {
        let max_turns = self.meta.frame_count.min(REPLAY_FRAME_CAPACITY - 1);
        let hill_record = self.meta.hill_record_marker.unwrap_or((0, 0));
        let checksum = replay_checksum(ReplayChecksumInput {
            start_x: self.meta.start_x,
            start_y: self.meta.start_y,
            max_turns,
            hill_idx: self.meta.hill_idx,
            hill_filename: self.meta.hill_filename.as_bytes(),
            hill_profile: self.meta.hill_profile,
            distance: self.meta.distance,
            flight_start: self.meta.flight_start,
            flight_stop: self.meta.flight_stop,
            hill_record_x: hill_record.0,
            hill_record_y: hill_record.1,
            author: self.meta.author.as_bytes(),
            start_gate_or_competition: self.meta.start_gate_or_competition,
        });

        let mut out = String::new();
        writeln!(&mut out, "{}", self.meta.start_x).expect("write string");
        writeln!(&mut out, "{}", self.meta.start_y).expect("write string");
        writeln!(&mut out, "{}", max_turns).expect("write string");
        writeln!(&mut out, "{}", self.meta.hill_idx).expect("write string");
        writeln!(&mut out, "{}", self.meta.hill_filename).expect("write string");
        writeln!(&mut out, "{}", self.meta.hill_profile).expect("write string");
        writeln!(&mut out, "{}", self.meta.snow_count).expect("write string");
        writeln!(&mut out, "{}", self.meta.distance).expect("write string");
        writeln!(&mut out, "{}", self.meta.flight_start).expect("write string");
        writeln!(&mut out, "{}", self.meta.flight_stop).expect("write string");
        writeln!(&mut out, "{}", hill_record.0).expect("write string");
        writeln!(&mut out, "{}", hill_record.1).expect("write string");
        writeln!(&mut out, "{}", self.meta.suit_color).expect("write string");
        writeln!(&mut out, "{}", self.meta.ski_color).expect("write string");
        writeln!(&mut out, "{}", self.meta.author).expect("write string");
        writeln!(&mut out, "{}", self.meta.name).expect("write string");
        writeln!(&mut out, "{}", self.meta.saved_at).expect("write string");
        writeln!(&mut out, "{}", i32::from(self.meta.has_bib)).expect("write string");
        writeln!(&mut out, "{}", self.meta.start_gate_or_competition).expect("write string");
        writeln!(&mut out, "{}", checksum).expect("write string");
        out.push_str("0\n\n--- Replay Data --- \n");

        let mut bytes = out.into_bytes();
        bytes.push(b'*');
        encode_frames(&mut bytes, &self.frames);
        bytes
    }
}

struct ReplayChecksumInput<'a> {
    start_x: i32,
    start_y: i32,
    max_turns: usize,
    hill_idx: usize,
    hill_filename: &'a [u8],
    hill_profile: i32,
    distance: i32,
    flight_start: usize,
    flight_stop: usize,
    hill_record_x: i32,
    hill_record_y: i32,
    author: &'a [u8],
    start_gate_or_competition: i32,
}

fn replay_checksum(input: ReplayChecksumInput<'_>) -> i32 {
    let mut check = 0_i32;
    check += smallint(input.start_x * 2 + input.start_y);
    check += smallint(input.max_turns as i32 * 3);
    if input.hill_idx <= NUM_WC_HILLS {
        check += smallint(input.hill_idx as i32 * 131);
    }
    check += word(valuestr(input.hill_filename, 3) as i32 * 3) as i32;
    check += input.hill_profile;
    check += smallint((input.distance + 2) * 69);
    check += smallint((input.flight_start + input.flight_stop) as i32);
    check += smallint((input.hill_record_x + input.hill_record_y) * 2);
    check += valuestr(input.author, 2) as i32;
    check += smallint(input.start_gate_or_competition * 1412);
    check ^ REPLAY_CHECK_XOR
}

fn valuestr(bytes: &[u8], arvo: i32) -> u16 {
    let mut word1 = 0_u16;
    for (idx, byte) in bytes.iter().enumerate() {
        let index = idx as u16 + 1;
        let multiplier = (index % 5) + 41;
        word1 = word1.wrapping_add(u16::from(*byte).wrapping_mul(multiplier));
    }
    word1
        .wrapping_mul((arvo.rem_euclid(7) as u16).wrapping_add(1))
        .wrapping_add(arvo as u16)
}

fn smallint(value: i32) -> i32 {
    i32::from(value as i16)
}

fn word(value: i32) -> u16 {
    value as u16
}

fn decode_frames(data: &[u8], max_turns: usize) -> Result<Vec<ReplayFrame>, ReplayError> {
    let expected = REPLAY_FRAME_CAPACITY * 5;
    if data.len() < expected {
        return Err(ReplayError::MissingFrameData {
            expected,
            actual: data.len(),
        });
    }
    let count = (max_turns + 1).min(REPLAY_FRAME_CAPACITY);
    let mut frames = Vec::with_capacity(count);
    for frame_idx in 0..count {
        let base = frame_idx * 5;
        frames.push(ReplayFrame {
            dx: (i32::from(data[base]) - 128) as i8,
            dy: (i32::from(data[base + 1]) - 128) as i8,
            body_anim: data[base + 2].saturating_sub(1),
            ski_anim: data[base + 3].saturating_sub(1),
            wind: (i32::from(data[base + 4]) - 128) as i8,
        });
    }
    Ok(frames)
}

fn encode_frames(out: &mut Vec<u8>, frames: &[ReplayFrame]) {
    for idx in 0..REPLAY_FRAME_CAPACITY {
        let frame = frames.get(idx).copied().unwrap_or(ReplayFrame {
            dx: 0,
            dy: 0,
            body_anim: 0,
            ski_anim: 0,
            wind: 0,
        });
        out.push((i16::from(frame.dx) + 128) as u8);
        out.push((i16::from(frame.dy) + 128) as u8);
        out.push(frame.body_anim + 1);
        out.push(frame.ski_anim + 1);
        out.push((i16::from(frame.wind) + 128) as u8);
    }
}

struct ReplayParser<'a> {
    data: &'a [u8],
    pos: usize,
    previous_raw_line: &'a [u8],
}

impl<'a> ReplayParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            previous_raw_line: &[],
        }
    }

    fn previous_raw_line(&self) -> &'a [u8] {
        self.previous_raw_line
    }

    fn string_line(&mut self, field: &'static str) -> Result<String, ReplayError> {
        let line = self.line(field)?;
        Ok(String::from_utf8_lossy(line).to_string())
    }

    fn i32_line(&mut self, field: &'static str) -> Result<i32, ReplayError> {
        let line = self.line(field)?;
        let value = String::from_utf8_lossy(line).trim().to_string();
        value
            .parse::<i32>()
            .map_err(|_| ReplayError::InvalidNumber { field, value })
    }

    fn usize_line(&mut self, field: &'static str) -> Result<usize, ReplayError> {
        let value = self.i32_line(field)?;
        usize::try_from(value).map_err(|_| ReplayError::InvalidNumber {
            field,
            value: value.to_string(),
        })
    }

    fn u16_line(&mut self, field: &'static str) -> Result<u16, ReplayError> {
        let value = self.i32_line(field)?;
        u16::try_from(value).map_err(|_| ReplayError::InvalidNumber {
            field,
            value: value.to_string(),
        })
    }

    fn u8_line(&mut self, field: &'static str) -> Result<u8, ReplayError> {
        let value = self.i32_line(field)?;
        u8::try_from(value).map_err(|_| ReplayError::InvalidNumber {
            field,
            value: value.to_string(),
        })
    }

    fn line(&mut self, field: &'static str) -> Result<&'a [u8], ReplayError> {
        if self.pos >= self.data.len() {
            return Err(ReplayError::MissingLine(field));
        }
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != b'\n' {
            self.pos += 1;
        }
        let mut end = self.pos;
        if self.pos < self.data.len() {
            self.pos += 1;
        }
        if end > start && self.data[end - 1] == b'\r' {
            end -= 1;
        }
        self.previous_raw_line = &self.data[start..end];
        Ok(self.previous_raw_line)
    }

    fn replay_data(&mut self) -> Result<&'a [u8], ReplayError> {
        while self.pos < self.data.len() {
            if self.data[self.pos] == b'*' {
                self.pos += 1;
                return Ok(&self.data[self.pos..]);
            }
            self.pos += 1;
        }
        Err(ReplayError::MissingReplayData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace() -> ReplayTrace {
        ReplayTrace {
            meta: ReplayMeta {
                start_x: 10,
                start_y: 20,
                hill_idx: 1,
                snow_count: 42,
                distance: 955,
                flight_start: 2,
                flight_stop: 4,
                hill_record_marker: Some((300, 120)),
                hill_filename: "HILLBASE".to_string(),
                hill_profile: 7,
                suit_color: 3,
                ski_color: 4,
                saved_at: "1.1.2000 00:00".to_string(),
                has_bib: false,
                author: "TESTER".to_string(),
                name: "ROUNDTRIP".to_string(),
                start_gate_or_competition: 85,
                frame_count: 4,
                checksum: 0,
                valid_checksum: true,
                intro: false,
            },
            frames: vec![
                ReplayFrame {
                    dx: 0,
                    dy: 0,
                    body_anim: 163,
                    ski_anim: 72,
                    wind: -3,
                },
                ReplayFrame {
                    dx: 2,
                    dy: -1,
                    body_anim: 164,
                    ski_anim: 73,
                    wind: 4,
                },
                ReplayFrame {
                    dx: -2,
                    dy: 1,
                    body_anim: 111,
                    ski_anim: 90,
                    wind: 0,
                },
                ReplayFrame {
                    dx: 3,
                    dy: 4,
                    body_anim: 112,
                    ski_anim: 91,
                    wind: 7,
                },
                ReplayFrame {
                    dx: 0,
                    dy: 0,
                    body_anim: 130,
                    ski_anim: 80,
                    wind: 1,
                },
            ],
        }
    }

    #[test]
    fn sjr_roundtrip_preserves_metadata_frames_and_checksum() {
        let bytes = trace().to_sjr_bytes();
        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("valid replay");

        assert!(parsed.meta.valid_checksum);
        assert_eq!(parsed.meta.start_x, 10);
        assert_eq!(parsed.meta.start_y, 20);
        assert_eq!(parsed.meta.hill_filename, "HILLBASE");
        assert_eq!(parsed.meta.frame_count, 4);
        assert_eq!(parsed.frames.len(), 5);
        assert_eq!(parsed.frames[1].dx, 2);
        assert_eq!(parsed.frames[1].dy, -1);
        assert_eq!(parsed.frames[3].body_anim, 112);
        assert_eq!(parsed.frames[3].wind, 7);
    }

    #[test]
    fn sjr_anim_indices_are_pascal_indexed_only_on_disk() {
        let bytes = trace().to_sjr_bytes();
        let data_start = bytes
            .iter()
            .position(|byte| *byte == b'*')
            .expect("replay data marker")
            + 1;

        assert_eq!(bytes[data_start + 2], 164);
        assert_eq!(bytes[data_start + 3], 73);

        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("valid replay");
        assert_eq!(parsed.frames[0].body_anim, 163);
        assert_eq!(parsed.frames[0].ski_anim, 72);
    }

    #[test]
    fn checksum_uses_pascal_wrapping_valuestr_bytes() {
        let input = ReplayChecksumInput {
            start_x: 10,
            start_y: 20,
            max_turns: 4,
            hill_idx: 1,
            hill_filename: b"HILLBASE",
            hill_profile: 7,
            distance: 955,
            flight_start: 2,
            flight_stop: 4,
            hill_record_x: 300,
            hill_record_y: 120,
            author: b"TESTER",
            start_gate_or_competition: 85,
        };
        assert_eq!(replay_checksum(input), 3_755_229);
    }

    #[test]
    fn recorder_frame_count_is_pascal_last_frame_index() {
        let mut recorder = ReplayRecorder::default();
        recorder.start(trace().meta);
        recorder.record_frame((10, 20), (11, 22), 163, 72, -2);
        recorder.record_frame((11, 22), (14, 25), 164, 73, -1);

        let trace = recorder.finish().expect("trace");
        let bytes = trace.to_sjr_bytes();
        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("valid replay");

        assert_eq!(trace.meta.frame_count, 1);
        assert_eq!(parsed.frames.len(), 2);
        assert_eq!(parsed.frames[1].body_anim, 164);
        assert_eq!(parsed.frames[1].ski_anim, 73);
    }

    #[test]
    fn recorder_ignores_frames_after_result_stop() {
        let mut recorder = ReplayRecorder::default();
        recorder.start(trace().meta);
        recorder.record_frame((10, 20), (11, 22), 163, 72, -2);
        recorder.stop();
        recorder.record_frame((11, 22), (11, 22), 136, 90, -2);

        let trace = recorder.finish().expect("trace");

        assert_eq!(trace.frames.len(), 1);
        assert_eq!(trace.frames[0].body_anim, 163);
    }
}
