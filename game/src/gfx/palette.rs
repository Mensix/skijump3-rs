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
    for (i, &fd) in fade.iter().enumerate() {
        let idx = target_base + i;
        palette.set(
            idx,
            [
                (fd * f32::from(suit[1])).round().min(63.0) as u8,
                (fd * f32::from(suit[2])).round().min(63.0) as u8,
                (fd * f32::from(suit[3])).round().min(63.0) as u8,
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

const REPLACE_MENU: [[u8; 3]; 12] = [
    [20, 20, 20],
    [26, 26, 26],
    [10, 10, 10],
    [15, 15, 15],
    [28, 8, 24],
    [34, 13, 28],
    [0, 24, 24],
    [6, 30, 30],
    [0, 25, 0],
    [5, 30, 5],
    [47, 0, 0],
    [54, 10, 10],
];

pub fn apply_menu_tint(palette: &mut Palette, index: usize, col: usize) {
    let col = col.min(5);
    let upper = &REPLACE_MENU[col * 2];
    let lower = &REPLACE_MENU[col * 2 + 1];
    palette.set(242 + index, *upper);
    palette.set(247 + index, *lower);
}

pub fn apply_logo_tint(palette: &mut Palette, col: usize) {
    const REPLACE_LOGO: [[u8; 3]; 4] = [
        [46, 46, 63],
        [32, 32, 63],
        [51, 51, 51],
        [38, 38, 38],
    ];
    let col = col.min(3);
    palette.set(253, REPLACE_LOGO[col * 2]);
    palette.set(254, REPLACE_LOGO[col * 2 + 1]);
}
