use engine::color::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rgb6(pub u8, pub u8, pub u8);

impl Rgb6 {
    const fn rgba(self) -> Rgba {
        Rgba::from_rgb6(self.0, self.1, self.2)
    }
}

pub const JUMPER_SUIT_SOURCE_SHADE_1: u8 = 216;
pub const JUMPER_SUIT_SOURCE_SHADE_3: u8 = 218;
pub const JUMPER_BIB_SOURCE_SHADE_1: u8 = 220;
pub const JUMPER_BIB_SOURCE_SHADE_3: u8 = 221;
pub const JUMPER_SKI_SOURCE: u8 = 231;

const JUMPER_BIB_SHADE_1: Rgb6 = Rgb6(49, 45, 0);
const JUMPER_BIB_SHADE_3: Rgb6 = Rgb6(34, 31, 0);

pub(crate) const SUIT_COLORS: [[u8; 4]; 8] = [
    [0, 53, 17, 53],
    [0, 55, 33, 11],
    [0, 11, 48, 18],
    [0, 24, 28, 63],
    [0, 63, 17, 17],
    [0, 33, 33, 33],
    [1, 10, 10, 10],
    [0, 45, 17, 63],
];

pub(crate) const SKI_COLORS: [Rgb6; 4] = [
    Rgb6(63, 63, 32),
    Rgb6(60, 60, 60),
    Rgb6(33, 60, 33),
    Rgb6(63, 43, 43),
];

const SUIT_FADE_DOWN: [f32; 4] = [1.0, 0.87, 0.75, 0.63];
const SUIT_FADE_UP: [f32; 4] = [1.0, 1.50, 2.00, 2.50];

pub fn suit_shade_rgba(col: usize) -> [[u8; 3]; 4] {
    let col = col.min(SUIT_COLORS.len() - 1);
    let suit = SUIT_COLORS[col];
    let fade = if suit[0] == 0 {
        SUIT_FADE_DOWN
    } else {
        SUIT_FADE_UP
    };
    let mut colors = [[0u8; 3]; 4];
    for (i, &fd) in fade.iter().enumerate() {
        colors[i] = [
            (fd * f32::from(suit[1])).round().min(63.0) as u8,
            (fd * f32::from(suit[2])).round().min(63.0) as u8,
            (fd * f32::from(suit[3])).round().min(63.0) as u8,
        ];
    }
    colors
}

pub fn suit_color_shade(col: usize, shade: usize) -> Rgba {
    let rgb = suit_shade_rgba(col)[shade];
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

pub fn jumper_bib_color_shade(shade: usize) -> Rgba {
    match shade {
        1 => JUMPER_BIB_SHADE_1.rgba(),
        3 => JUMPER_BIB_SHADE_3.rgba(),
        _ => JUMPER_BIB_SHADE_1.rgba(),
    }
}

pub fn ski_color(col: usize) -> Rgba {
    let col = col.min(SKI_COLORS.len() - 1);
    SKI_COLORS[col].rgba()
}

pub fn suit_color_shade_rgb(rgb: [u8; 3], shade: usize) -> Rgba {
    let fd = SUIT_FADE_DOWN[shade.min(3)];
    Rgba::from_rgb6(
        (fd * f32::from(rgb[0])).round().min(63.0) as u8,
        (fd * f32::from(rgb[1])).round().min(63.0) as u8,
        (fd * f32::from(rgb[2])).round().min(63.0) as u8,
    )
}

pub fn ski_color_rgb(rgb: [u8; 3]) -> Rgba {
    Rgba::from_rgb6(rgb[0].min(63), rgb[1].min(63), rgb[2].min(63))
}

pub fn suit_palette_rgb(idx: usize) -> [u8; 3] {
    let s = SUIT_COLORS[idx.min(SUIT_COLORS.len() - 1)];
    [s[1], s[2], s[3]]
}

pub fn ski_palette_rgb(idx: usize) -> [u8; 3] {
    let c = SKI_COLORS[idx.min(SKI_COLORS.len() - 1)];
    [c.0, c.1, c.2]
}
