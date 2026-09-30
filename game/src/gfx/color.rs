use engine::color::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rgb6(pub [u8; 3]);

impl Rgb6 {
    pub const fn to_rgba(self) -> Rgba {
        Rgba::from_rgb6(self.0[0], self.0[1], self.0[2])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shade {
    S0,
    S1,
    S2,
    S3,
}
