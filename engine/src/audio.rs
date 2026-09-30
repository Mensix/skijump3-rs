use sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};

const SAMPLE_RATE: i32 = 44_100;
const BEEP_DURATION_MS: usize = 110;
const BEEP_VOLUME: f32 = 0.18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beep {
    Type1,
    Type2,
}

impl Beep {
    const fn frequency(self) -> f32 {
        match self {
            Self::Type1 => 1_000.0,
            Self::Type2 => 2_000.0,
        }
    }
}

pub struct TonePlayer {
    stream: Option<AudioStreamOwner>,
    buffers: [Box<[f32]>; 2],
}

impl TonePlayer {
    pub fn new(sdl: &sdl3::Sdl) -> Self {
        let stream = (|| {
            let audio = sdl.audio().ok()?;
            let spec = AudioSpec {
                freq: Some(SAMPLE_RATE),
                channels: Some(1),
                format: Some(AudioFormat::f32_sys()),
            };
            let stream = audio
                .default_playback_device()
                .open_device_stream(Some(&spec))
                .ok()?;
            let _ = stream.resume();
            Some(stream)
        })();
        Self {
            stream,
            buffers: beep_buffers(),
        }
    }

    pub fn play(&mut self, beep: Beep) {
        let samples = match beep {
            Beep::Type1 => &self.buffers[0],
            Beep::Type2 => &self.buffers[1],
        };
        if let Some(stream) = self.stream.as_mut() {
            let _ = stream.put_data_f32(samples);
        }
    }
}

fn beep_buffers() -> [Box<[f32]>; 2] {
    [
        generate_beep(Beep::Type1).into_boxed_slice(),
        generate_beep(Beep::Type2).into_boxed_slice(),
    ]
}

fn generate_beep(beep: Beep) -> Vec<f32> {
    let sample_count = SAMPLE_RATE as usize * BEEP_DURATION_MS / 1_000;
    let samples_per_cycle = SAMPLE_RATE as f32 / beep.frequency();
    (0..sample_count)
        .map(|sample| {
            if (sample as f32 % samples_per_cycle) < samples_per_cycle / 2.0 {
                BEEP_VOLUME
            } else {
                -BEEP_VOLUME
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rising_edges(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] > 0.0)
            .count()
    }

    #[test]
    fn original_beeps_have_expected_duration_and_frequencies() {
        let type1 = generate_beep(Beep::Type1);
        let type2 = generate_beep(Beep::Type2);

        assert_eq!(type1.len(), 4_851);
        assert_eq!(type2.len(), 4_851);
        assert!((109..=110).contains(&rising_edges(&type1)));
        assert!((219..=220).contains(&rising_edges(&type2)));
    }

    #[test]
    fn generated_beeps_are_bounded_square_waves() {
        for sample in generate_beep(Beep::Type1) {
            assert!(sample == BEEP_VOLUME || sample == -BEEP_VOLUME);
        }
    }
}
