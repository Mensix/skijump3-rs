use crate::data::hill::{HillInfo, FALLBACK_HILL_FILENAME};
use crate::data::hill_profile::HillTerrain;
use crate::gfx::jumper_colors::{SkiIdx, SuitIdx};
use crate::jump::animation::select_jumper_sprites;
use crate::jump::config::JumpConfig;
use crate::jump::math;
use crate::jump::state::JumpState;
use crate::jump::types::{FlightWind, JumpPhase};
use crate::text::encoding;
use std::fmt::Write;
use std::str;

const REPLAY_FRAME_CAPACITY: usize = 1001;
const REPLAY_CHECK_XOR: i32 = 3_675_433;
const NUM_WC_HILLS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayParseError {
    Truncated,
    InvalidUtf8(&'static str),
    InvalidValue(&'static str),
}

impl std::fmt::Display for ReplayParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => f.write_str("Replay is truncated."),
            Self::InvalidUtf8(field) => write!(f, "Invalid text in {field}."),
            Self::InvalidValue(field) => write!(f, "Invalid value for {field}."),
        }
    }
}

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
    pub suit_color: SuitIdx,
    pub ski_color: SkiIdx,
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

    pub const fn stop(&mut self) {
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

    pub const fn mark_flight_start(&mut self) {
        if let Some(meta) = &mut self.meta {
            if meta.flight_start == 0 {
                meta.flight_start = self.frames.len();
            }
        }
    }

    pub const fn mark_flight_stop(&mut self) {
        if let Some(meta) = &mut self.meta {
            meta.flight_stop = self.frames.len();
        }
    }

    pub const fn set_distance(&mut self, distance: i32) {
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

#[derive(Debug, Clone, Default)]
pub(crate) struct LiveReplayRecorder {
    recorder: ReplayRecorder,
    previous_pos: Option<(i32, i32)>,
}

impl LiveReplayRecorder {
    pub(crate) fn new(
        config: &JumpConfig,
        state: Option<&JumpState>,
        hill_record_marker: Option<(i32, i32)>,
    ) -> Self {
        let mut tracker = Self::default();
        tracker.reset(config, state, hill_record_marker);
        tracker
    }

    pub(crate) fn reset(
        &mut self,
        config: &JumpConfig,
        state: Option<&JumpState>,
        hill_record_marker: Option<(i32, i32)>,
    ) {
        self.recorder = ReplayRecorder::default();
        self.previous_pos = state.map(|state| (state.x, state.y));
        if let Some(state) = state {
            self.recorder
                .start(Self::meta(config, state, hill_record_marker));
        }
    }

    pub(crate) fn on_phase_change(
        &mut self,
        previous_phase: JumpPhase,
        current_phase: JumpPhase,
        state: &JumpState,
    ) {
        if previous_phase != JumpPhase::Flight && current_phase == JumpPhase::Flight {
            self.recorder.mark_flight_start();
        }
        if previous_phase == JumpPhase::Flight && current_phase == JumpPhase::Landing {
            self.recorder.mark_flight_stop();
        }
        if matches!(
            current_phase,
            JumpPhase::Landing | JumpPhase::Result | JumpPhase::Disqualified
        ) {
            self.recorder
                .set_distance(math::round(state.distance * 10.0));
        }
    }

    pub(crate) fn record_frame(
        &mut self,
        terrain: &HillTerrain,
        state: &JumpState,
        wind: FlightWind,
    ) {
        if self
            .recorder
            .meta
            .as_ref()
            .is_some_and(|meta| meta.start_gate_or_competition > 5)
        {
            self.set_start_gate(state.start_gate);
        }
        if matches!(state.phase, JumpPhase::Result | JumpPhase::Disqualified) {
            self.recorder.stop();
            return;
        }

        let current_pos = (state.x, state.y);
        let previous_pos = self.previous_pos.unwrap_or(current_pos);
        let sprites = select_jumper_sprites(state.animation_context(terrain));
        self.recorder.record_frame(
            previous_pos,
            current_pos,
            sprites.body,
            sprites.skis,
            wind.value,
        );
        self.previous_pos = Some(current_pos);
    }

    pub(crate) fn trace(&self) -> Option<ReplayTrace> {
        self.recorder.finish()
    }

    pub(crate) fn set_has_bib(&mut self, has_bib: bool) {
        if let Some(meta) = &mut self.recorder.meta {
            meta.has_bib = has_bib;
        }
    }

    pub(crate) fn set_start_gate(&mut self, start_gate: i32) {
        if let Some(meta) = &mut self.recorder.meta {
            meta.start_gate_or_competition = 100 - start_gate;
        }
    }

    fn meta(
        config: &JumpConfig,
        state: &JumpState,
        hill_record_marker: Option<(i32, i32)>,
    ) -> ReplayMeta {
        ReplayMeta {
            start_x: state.x,
            start_y: state.y,
            hill_idx: config.hill_idx,
            snow_count: config.snow_count,
            distance: 0,
            flight_start: 0,
            flight_stop: 0,
            hill_record_marker,
            hill_filename: replay_hill_filename(config.hill.as_ref(), config.is_custom_hill),
            hill_profile: config
                .hill
                .as_ref()
                .and_then(|hill| i32::try_from(hill.profile_checksum).ok())
                .unwrap_or_default(),
            suit_color: SuitIdx::from_rgb(config.participant.suit_color),
            ski_color: SkiIdx::from_rgb(config.participant.ski_color),
            saved_at: String::new(),
            has_bib: false,
            author: String::new(),
            name: config.participant.display_name().to_string(),
            start_gate_or_competition: replay_competition_value(
                config.replay_competition_code,
                config.start_gate,
            ),
            frame_count: 0,
            checksum: 0,
            valid_checksum: true,
            intro: false,
        }
    }
}

fn replay_hill_filename(hill: Option<&HillInfo>, is_custom_hill: bool) -> String {
    match (hill, is_custom_hill) {
        (Some(hill), true) => hill.record_key.clone(),
        _ => FALLBACK_HILL_FILENAME.to_string(),
    }
}

const fn replay_competition_value(competition_code: Option<i32>, start_gate: i32) -> i32 {
    match competition_code {
        Some(code) => code,
        None => 100 - start_gate,
    }
}

impl ReplayTrace {
    pub fn from_sjr_bytes(data: &[u8], intro: bool) -> Result<Self, ReplayParseError> {
        let mut parser = ReplayParser::new(data);
        let start_x = parser.number("start X")?;
        let start_y = parser.number("start Y")?;
        let max_turns: usize = parser.number("frame count")?;
        if max_turns >= REPLAY_FRAME_CAPACITY {
            return Err(ReplayParseError::InvalidValue("frame count"));
        }
        let hill_idx_raw: usize = parser.number("hill index")?;
        let hill_idx = hill_idx_raw.saturating_sub(1);
        let hill_filename = parser.text("hill filename")?.to_string();
        let hill_filename_raw = parser.previous_raw_line().to_vec();
        let hill_profile = parser.number("hill profile")?;
        let snow_count = parser.number("snow count")?;
        let distance = parser.number("distance")?;
        let flight_start = parser.number("flight start")?;
        let flight_stop = parser.number("flight stop")?;
        let hr_x = parser.number("record X")?;
        let hr_y = parser.number("record Y")?;
        let suit_color = SuitIdx(parser.number("suit color")?);
        let ski_color = SkiIdx(parser.number("ski color")?);
        let author_raw = parser.next_line()?.to_vec();
        let author = encoding::decode(&author_raw);
        let name_raw = parser.next_line()?.to_vec();
        let name = encoding::decode(&name_raw);
        let saved_at = parser.text("date")?.to_string();
        let has_bib = parser.number::<i32>("bib")? != 0;
        let start_gate_or_competition = parser.number("competition")?;
        let checksum = parser.number("checksum")?;
        parser.next_line()?;
        let replay_data = parser.skip_to_replay_data()?;
        let required_bytes = REPLAY_FRAME_CAPACITY * 5;
        if replay_data.len() < required_bytes {
            return Err(ReplayParseError::Truncated);
        }
        let frames = decode_frames(replay_data, max_turns);
        let expected_checksum = replay_checksum(ReplayChecksumInput {
            start_x,
            start_y,
            max_turns,
            hill_idx: hill_idx_raw,
            hill_filename: &hill_filename_raw,
            hill_profile,
            distance,
            flight_start,
            flight_stop,
            hill_record_x: hr_x,
            hill_record_y: hr_y,
            author: &author_raw,
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
        let file_hill_idx = self.meta.hill_idx.wrapping_add(1);
        let author = encoding::encode(&self.meta.author);
        let name = encoding::encode(&self.meta.name);
        let checksum = replay_checksum(ReplayChecksumInput {
            start_x: self.meta.start_x,
            start_y: self.meta.start_y,
            max_turns,
            hill_idx: file_hill_idx,
            hill_filename: self.meta.hill_filename.as_bytes(),
            hill_profile: self.meta.hill_profile,
            distance: self.meta.distance,
            flight_start: self.meta.flight_start,
            flight_stop: self.meta.flight_stop,
            hill_record_x: hill_record.0,
            hill_record_y: hill_record.1,
            author: &author,
            start_gate_or_competition: self.meta.start_gate_or_competition,
        });

        let mut out = String::new();
        let _ = writeln!(&mut out, "{}", self.meta.start_x);
        let _ = writeln!(&mut out, "{}", self.meta.start_y);
        let _ = writeln!(&mut out, "{max_turns}");
        let _ = writeln!(&mut out, "{file_hill_idx}");
        let _ = writeln!(&mut out, "{}", self.meta.hill_filename);
        let _ = writeln!(&mut out, "{}", self.meta.hill_profile);
        let _ = writeln!(&mut out, "{}", self.meta.snow_count);
        let _ = writeln!(&mut out, "{}", self.meta.distance);
        let _ = writeln!(&mut out, "{}", self.meta.flight_start);
        let _ = writeln!(&mut out, "{}", self.meta.flight_stop);
        let _ = writeln!(&mut out, "{}", hill_record.0);
        let _ = writeln!(&mut out, "{}", hill_record.1);
        let _ = writeln!(&mut out, "{}", self.meta.suit_color.0);
        let _ = writeln!(&mut out, "{}", self.meta.ski_color.0);
        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(&author);
        bytes.push(b'\n');
        bytes.extend_from_slice(&name);
        bytes.push(b'\n');
        let mut out = String::new();
        let _ = writeln!(&mut out, "{}", self.meta.saved_at);
        let _ = writeln!(&mut out, "{}", i32::from(self.meta.has_bib));
        let _ = writeln!(&mut out, "{}", self.meta.start_gate_or_competition);
        let _ = writeln!(&mut out, "{checksum}");
        out.push_str("0\n\n--- Replay Data --- \n");

        bytes.extend_from_slice(out.as_bytes());
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
    check += i32::from(word(i32::from(str_checksum(input.hill_filename, 3)) * 3));
    check += input.hill_profile;
    check += smallint((input.distance + 2) * 69);
    check += smallint((input.flight_start + input.flight_stop) as i32);
    check += smallint((input.hill_record_x + input.hill_record_y) * 2);
    check += i32::from(str_checksum(input.author, 2));
    check += smallint(input.start_gate_or_competition * 1412);
    check ^ REPLAY_CHECK_XOR
}

fn str_checksum(bytes: &[u8], value: i32) -> u16 {
    let mut word1 = 0_u16;
    for (idx, byte) in bytes.iter().enumerate() {
        let index = idx as u16 + 1;
        let multiplier = (index % 5) + 41;
        word1 = word1.wrapping_add(u16::from(*byte).wrapping_mul(multiplier));
    }
    word1
        .wrapping_mul((value.rem_euclid(7) as u16).wrapping_add(1))
        .wrapping_add(value as u16)
}

fn smallint(value: i32) -> i32 {
    i32::from(value as i16)
}

const fn word(value: i32) -> u16 {
    value as u16
}

fn decode_frames(data: &[u8], max_turns: usize) -> Vec<ReplayFrame> {
    let count = (max_turns + 1).min(REPLAY_FRAME_CAPACITY);
    data.chunks_exact(5)
        .take(count)
        .map(|chunk| ReplayFrame {
            dx: (i32::from(chunk[0]) - 128) as i8,
            dy: (i32::from(chunk[1]) - 128) as i8,
            body_anim: chunk[2].saturating_sub(1),
            ski_anim: chunk[3].saturating_sub(1),
            wind: (i32::from(chunk[4]) - 128) as i8,
        })
        .collect()
}

fn encode_frames(out: &mut Vec<u8>, frames: &[ReplayFrame]) {
    let empty = ReplayFrame {
        dx: 0,
        dy: 0,
        body_anim: 0,
        ski_anim: 0,
        wind: 0,
    };
    for frame in frames
        .iter()
        .chain(std::iter::repeat(&empty))
        .take(REPLAY_FRAME_CAPACITY)
    {
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
    const fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            previous_raw_line: &[],
        }
    }

    const fn previous_raw_line(&self) -> &'a [u8] {
        self.previous_raw_line
    }

    fn next_line(&mut self) -> Result<&'a [u8], ReplayParseError> {
        if self.pos >= self.data.len() {
            return Err(ReplayParseError::Truncated);
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

    fn text(&mut self, field: &'static str) -> Result<&'a str, ReplayParseError> {
        str::from_utf8(self.next_line()?).map_err(|_| ReplayParseError::InvalidUtf8(field))
    }

    fn number<T>(&mut self, field: &'static str) -> Result<T, ReplayParseError>
    where
        T: str::FromStr,
    {
        self.text(field)?
            .trim()
            .parse()
            .map_err(|_| ReplayParseError::InvalidValue(field))
    }

    fn skip_to_replay_data(&mut self) -> Result<&'a [u8], ReplayParseError> {
        while self.pos < self.data.len() {
            if self.data[self.pos] == b'*' {
                self.pos += 1;
                return Ok(&self.data[self.pos..]);
            }
            self.pos += 1;
        }
        Err(ReplayParseError::Truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::sprites::Sprite;

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
                suit_color: SuitIdx(3),
                ski_color: SkiIdx(4),
                saved_at: "SAT 01 JAN 2000, 00:00".to_string(),
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
                    body_anim: Sprite::IdleBody as u8,
                    ski_anim: 72,
                    wind: -3,
                },
                ReplayFrame {
                    dx: 2,
                    dy: -1,
                    body_anim: Sprite::InrunTransition as u8,
                    ski_anim: 73,
                    wind: 4,
                },
                ReplayFrame {
                    dx: -2,
                    dy: 1,
                    body_anim: Sprite::TakeoffArmsUp as u8,
                    ski_anim: 90,
                    wind: 0,
                },
                ReplayFrame {
                    dx: 3,
                    dy: 4,
                    body_anim: Sprite::LandingLoopBase as u8,
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
        assert_eq!(parsed.meta.hill_profile, 7);
        assert!(!parsed.meta.has_bib);
        assert_eq!(parsed.meta.start_gate_or_competition, 85);
        assert_eq!(parsed.meta.frame_count, 4);
        assert_eq!(parsed.frames.len(), 5);
        assert_eq!(parsed.frames[1].dx, 2);
        assert_eq!(parsed.frames[1].dy, -1);
        assert_eq!(parsed.frames[3].body_anim, Sprite::LandingLoopBase as u8);
        assert_eq!(parsed.frames[3].wind, 7);
    }

    #[test]
    fn sjr_roundtrip_writes_cp850_names_and_checksums_encoded_author() {
        let mut trace = trace();
        trace.meta.author = "Jürgen Øst".to_string();
        trace.meta.name = "Mäkihyppy".to_string();

        let bytes = trace.to_sjr_bytes();
        let header_lines: Vec<&[u8]> = bytes.split(|&byte| byte == b'\n').collect();
        assert_eq!(header_lines[14], encoding::encode("Jürgen Øst"));
        assert_eq!(header_lines[15], encoding::encode("Mäkihyppy"));

        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("valid replay");
        assert_eq!(parsed.meta.author, "Jürgen Øst");
        assert_eq!(parsed.meta.name, "Mäkihyppy");
        assert!(parsed.meta.valid_checksum);
    }

    #[test]
    fn custom_hill_bib_and_competition_metadata_roundtrip() {
        let mut trace = trace();
        trace.meta.hill_idx = 27;
        trace.meta.hill_filename = "custom-file:main".to_string();
        trace.meta.hill_profile = 1_234_567;
        trace.meta.has_bib = true;
        trace.meta.start_gate_or_competition = 4;

        let parsed = ReplayTrace::from_sjr_bytes(&trace.to_sjr_bytes(), false).expect("replay");

        assert_eq!(parsed.meta.hill_filename, "custom-file:main");
        assert_eq!(parsed.meta.hill_profile, 1_234_567);
        assert!(parsed.meta.has_bib);
        assert_eq!(parsed.meta.start_gate_or_competition, 4);
        assert!(parsed.meta.valid_checksum);
    }

    #[test]
    fn recorder_metadata_uses_builtin_and_custom_hill_keys_and_pascal_values() {
        let custom = HillInfo {
            record_key: "custom-file:main".to_string(),
            ..HillInfo::default()
        };

        assert_eq!(replay_hill_filename(Some(&custom), false), "HILLBASE");
        assert_eq!(
            replay_hill_filename(Some(&custom), true),
            "custom-file:main"
        );
        assert_eq!(replay_competition_value(None, 15), 85);
        assert_eq!(replay_competition_value(Some(4), 15), 4);
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
        assert_eq!(parsed.frames[0].body_anim, Sprite::IdleBody as u8);
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
        recorder.record_frame((10, 20), (11, 22), Sprite::IdleBody as u16, 72, -2);
        recorder.record_frame((11, 22), (14, 25), Sprite::InrunTransition as u16, 73, -1);

        let trace = recorder.finish().expect("trace");
        let bytes = trace.to_sjr_bytes();
        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("valid replay");

        assert_eq!(trace.meta.frame_count, 1);
        assert_eq!(parsed.frames.len(), 2);
        assert_eq!(parsed.frames[1].body_anim, Sprite::InrunTransition as u8);
        assert_eq!(parsed.frames[1].ski_anim, 73);
    }

    #[test]
    fn recorder_ignores_frames_after_result_stop() {
        let mut recorder = ReplayRecorder::default();
        recorder.start(trace().meta);
        recorder.record_frame((10, 20), (11, 22), Sprite::IdleBody as u16, 72, -2);
        recorder.stop();
        recorder.record_frame((11, 22), (11, 22), Sprite::LandingSlide as u16, 90, -2);

        let trace = recorder.finish().expect("trace");

        assert_eq!(trace.frames.len(), 1);
        assert_eq!(trace.frames[0].body_anim, Sprite::IdleBody as u8);
    }

    #[test]
    fn parser_rejects_truncated_replay() {
        let mut bytes = trace().to_sjr_bytes();
        bytes.truncate(bytes.len() - 1);

        assert_eq!(
            ReplayTrace::from_sjr_bytes(&bytes, false),
            Err(ReplayParseError::Truncated)
        );
    }

    #[test]
    fn parser_rejects_malformed_and_non_utf8_headers() {
        assert!(matches!(
            ReplayTrace::from_sjr_bytes(b"not-a-number\n", false),
            Err(ReplayParseError::InvalidValue("start X"))
        ));

        let mut bytes = trace().to_sjr_bytes();
        bytes[0] = 0xff;
        assert!(matches!(
            ReplayTrace::from_sjr_bytes(&bytes, false),
            Err(ReplayParseError::InvalidUtf8("start X"))
        ));
    }

    #[test]
    fn checksum_mismatch_is_reported_without_rejecting_metadata() {
        let mut bytes = trace().to_sjr_bytes();
        let marker = bytes.iter().position(|&byte| byte == b'*').unwrap();
        let header = std::str::from_utf8(&bytes[..marker]).unwrap();
        let checksum = header.lines().nth(19).unwrap();
        let checksum_offset = header
            .lines()
            .take(19)
            .map(|line| line.len() + 1)
            .sum::<usize>();
        bytes[checksum_offset] = if checksum.as_bytes()[0] == b'1' {
            b'2'
        } else {
            b'1'
        };

        let parsed = ReplayTrace::from_sjr_bytes(&bytes, false).expect("structurally valid replay");
        assert!(!parsed.meta.valid_checksum);
    }
}
