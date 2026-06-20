const MT_N: usize = 624;
const MT_M: usize = 397;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;
const MATRIX_A: u32 = 0x9908_b0df;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Random {
    state: [u32; MT_N],
    index: usize,
}

#[derive(Serialize, Deserialize)]
struct RandomSave {
    state: Vec<u32>,
    index: usize,
}

impl Serialize for Random {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RandomSave {
            state: self.state.to_vec(),
            index: self.index,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Random {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let save = RandomSave::deserialize(deserializer)?;
        let mut state = [0; MT_N];
        if save.state.len() != MT_N {
            return Err(serde::de::Error::custom("invalid RNG state length"));
        }
        state.copy_from_slice(&save.state);
        Ok(Self {
            state,
            index: save.index.min(MT_N),
        })
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Random {
    #[must_use]
    pub fn new(seed: u32) -> Self {
        let mut rng = Self {
            state: [0; MT_N],
            index: MT_N,
        };
        rng.set_seed(seed);
        rng
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.state[0] = seed;
        for i in 1..MT_N {
            self.state[i] = 1_812_433_253u32
                .wrapping_mul(self.state[i - 1] ^ (self.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        self.index = MT_N;
    }

    pub fn random_i32(&mut self, upper: i32) -> i32 {
        let mut limit = upper;
        if limit < 0 {
            limit += 1;
        }
        ((i64::from(self.next_u32()) * i64::from(limit)) >> 32) as i32
    }

    fn next_u32(&mut self) -> u32 {
        if self.index == MT_N {
            self.update_state();
        }

        let mut value = self.state[self.index];
        self.index += 1;

        value ^= value >> 11;
        value ^= (value << 7) & 0x9d2c_5680;
        value ^= (value << 15) & 0xefc6_0000;
        value ^ (value >> 18)
    }

    fn update_state(&mut self) {
        for i in 0..(MT_N - MT_M) {
            self.state[i] = self.state[i + MT_M] ^ twist(self.state[i], self.state[i + 1]);
        }
        for i in (MT_N - MT_M)..(MT_N - 1) {
            self.state[i] = self.state[i + MT_M - MT_N] ^ twist(self.state[i], self.state[i + 1]);
        }
        self.state[MT_N - 1] = self.state[MT_M - 1] ^ twist(self.state[MT_N - 1], self.state[0]);
        self.index = 0;
    }
}

const fn twist(u: u32, v: u32) -> u32 {
    (((u & UPPER_MASK) | (v & LOWER_MASK)) >> 1) ^ (0u32.wrapping_sub(v & 1) & MATRIX_A)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_fpc_random_i32_for_seed_zero() {
        let mut rng = Random::new(0);
        let actual: Vec<i32> = (0..10).map(|_| rng.random_i32(1_000_000)).collect();
        assert_eq!(
            actual,
            vec![
                548_813, 592_844, 715_189, 844_265, 602_763, 857_945, 544_883, 847_251, 423_654,
                623_563,
            ]
        );
    }

    #[test]
    fn matches_fpc_random_i32_for_seed_one() {
        let mut rng = Random::new(1);
        let actual: Vec<i32> = (0..10).map(|_| rng.random_i32(1_000_000)).collect();
        assert_eq!(
            actual,
            vec![
                417_021, 997_184, 720_324, 932_557, 114, 128_124, 302_332, 999_040, 146_755,
                236_088,
            ]
        );
    }

    #[test]
    fn matches_fpc_random_i32_for_static_mt_seed() {
        let mut rng = Random::new(5489);
        let actual: Vec<i32> = (0..10).map(|_| rng.random_i32(1_000_000)).collect();
        assert_eq!(
            actual,
            vec![
                814_723, 135_477, 905_791, 835_008, 126_986, 968_867, 913_375, 221_034, 632_359,
                308_167,
            ]
        );
    }

    #[test]
    fn verify_fpc_longint_overload_three_values() {
        let mut rng = Random::new(0);
        assert_eq!(rng.random_i32(180), 98);
        assert_eq!(rng.random_i32(120), 71);
        assert_eq!(rng.random_i32(142), 101);
        assert_eq!(rng.random_i32(50), 42);
        assert_eq!(rng.random_i32(2), 1);
    }
}
