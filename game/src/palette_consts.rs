use engine::palette::Palette;

pub const FONT_DEFAULT: u8 = 240;
pub const FONT_HEADER: u8 = 246;
pub const FONT_GOLD: u8 = 246;
pub const FONT_GREET: u8 = 247;
pub const FONT_NAME: u8 = 240;
pub const FONT_NEW: u8 = 246;
pub const FONT_BACK: u8 = 240;
pub const FONT_HELP: u8 = 241;
pub const BG_ERASE: u8 = 8;
pub const BG_LIST: u8 = 8;
pub const BG_LEFT: u8 = 243;
pub const BG_RIGHT: u8 = 244;
pub const BG_ORDER: u8 = 243;
pub const BG_MENU: u8 = 243;
pub const BG_PANEL: u8 = 244;

pub const SUIT_PALETTE_BASE: usize = 215;
pub const SKI_PALETTE_INDEX: usize = 231;

const SUIT_COLORS: [[u8; 4]; 8] = [
    [0, 53, 17, 53],
    [0, 55, 33, 11],
    [0, 11, 48, 18],
    [0, 24, 28, 63],
    [0, 63, 17, 17],
    [0, 33, 33, 33],
    [1, 10, 10, 10],
    [0, 45, 17, 63],
];

const SKI_COLORS: [[u8; 3]; 4] = [[63, 63, 32], [60, 60, 60], [33, 60, 33], [63, 43, 43]];

const SUIT_FADE_DOWN: [f32; 4] = [1.0, 0.87, 0.75, 0.63];
const SUIT_FADE_UP: [f32; 4] = [1.0, 1.50, 2.00, 2.50];

pub fn apply_suit_palette_at(palette: &mut Palette, col: usize, target_base: usize) {
    let col = col.min(SUIT_COLORS.len() - 1);
    let suit = SUIT_COLORS[col];
    let fade = if suit[0] == 0 {
        SUIT_FADE_DOWN
    } else {
        SUIT_FADE_UP
    };
    for i in 0..4 {
        let idx = target_base + i;
        palette.set(
            idx,
            [
                (fade[i] * suit[1] as f32).round().min(63.0) as u8,
                (fade[i] * suit[2] as f32).round().min(63.0) as u8,
                (fade[i] * suit[3] as f32).round().min(63.0) as u8,
            ],
        );
    }
}

pub fn apply_suit_palette(palette: &mut Palette, col: usize) {
    apply_suit_palette_at(palette, col, SUIT_PALETTE_BASE);
}

pub fn apply_ski_palette_at(palette: &mut Palette, col: usize, target: usize) {
    let col = col.min(SKI_COLORS.len() - 1);
    palette.set(target, SKI_COLORS[col]);
}

pub fn apply_ski_palette(palette: &mut Palette, col: usize) {
    apply_ski_palette_at(palette, col, SKI_PALETTE_INDEX);
}
