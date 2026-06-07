use engine::color::Rgba;
use engine::palette::Palette;

pub const UI_PALETTE_BASE: usize = 216;

pub const STANDARD_UI_PALETTE: [[u8; 3]; 40] = [
    [53, 17, 53],
    [63, 0, 0],
    [43, 12, 43],
    [63, 0, 0],
    [49, 45, 0],
    [34, 31, 0],
    [63, 0, 0],
    [56, 54, 54],
    [63, 63, 21],
    [54, 52, 10],
    [42, 42, 42],
    [42, 20, 10],
    [21, 21, 21],
    [57, 45, 38],
    [63, 0, 0],
    [63, 63, 32],
    [40, 40, 41],
    [48, 48, 49],
    [55, 55, 56],
    [63, 63, 63],
    [56, 13, 13],
    [13, 53, 13],
    [23, 23, 63],
    [63, 23, 23],
    [63, 63, 63],
    [44, 44, 44],
    [0, 0, 0],
    [18, 13, 34],
    [34, 13, 18],
    [20, 20, 20],
    [63, 57, 9],
    [9, 57, 63],
    [23, 16, 43],
    [43, 16, 23],
    [26, 26, 26],
    [52, 47, 0],
    [0, 47, 52],
    [51, 51, 51],
    [38, 38, 38],
    [63, 63, 63],
];

// ---------------------------------------------------------------------------
// RGBA UI color constants (migrated from palette-index legacy)
// ---------------------------------------------------------------------------

pub const FONT_DEFAULT: Rgba = Rgba::from_rgb6(63, 63, 63);
pub const FONT_HEADER: Rgba = Rgba::from_rgb6(63, 57, 9);
pub const FONT_GOLD: Rgba = FONT_HEADER;
pub const FONT_GREET: Rgba = Rgba::from_rgb6(9, 57, 63);
pub const FONT_NAME: Rgba = FONT_DEFAULT;
pub const FONT_NEW: Rgba = FONT_HEADER;
pub const FONT_BACK: Rgba = FONT_DEFAULT;
pub const FONT_HELP: Rgba = Rgba::from_rgb6(44, 44, 44);
pub const BG_ERASE: Rgba = Rgba::from_rgb6(5, 8, 20);
pub const BG_LIST: Rgba = BG_ERASE;
pub const BG_LEFT: Rgba = Rgba::from_rgb6(34, 13, 18);
pub const BG_RIGHT: Rgba = Rgba::from_rgb6(20, 20, 20);
pub const BG_ORDER: Rgba = BG_LEFT;

// Bright variants for dither overlay (old palette index + 5).
// Used in a later migration milestone — keep for now.
#[allow(dead_code)]
pub const BG_LEFT_BRIGHT: Rgba = Rgba::from_rgb6(23, 16, 43);
#[allow(dead_code)]
pub const BG_DITHER_BRIGHT: Rgba = Rgba::from_rgb6(26, 26, 26);
pub const BG_RIGHT_BRIGHT: Rgba = Rgba::from_rgb6(43, 16, 23);

// Additional fill/text colours from old palette indices
pub const FILL_BORDER: Rgba = Rgba::from_rgb6(23, 16, 43); // 248
pub const FILL_HIGHLIGHT: Rgba = Rgba::from_rgb6(52, 47, 0); // 251
pub const FILL_TURQUOISE: Rgba = Rgba::from_rgb6(0, 47, 52); // 252
pub const FILL_LINE: Rgba = Rgba::from_rgb6(5, 8, 22); // 9
pub const FILL_DIM: Rgba = Rgba::from_rgb6(20, 20, 20); // 244/245
pub const BLACK: Rgba = Rgba::rgb(0, 0, 0);

/// Return the brightened overlay colour for a dither-eligible fill colour.
/// Kept for a later migration milestone.
#[must_use]
#[allow(dead_code)]
pub fn brighten_fill_color(color: Rgba) -> Rgba {
    if color == BG_LEFT {
        BG_LEFT_BRIGHT
    } else if color == BG_RIGHT || color == BG_ORDER {
        BG_RIGHT_BRIGHT
    } else {
        BG_DITHER_BRIGHT
    }
}

/// True when `color` is a dither-eligible fill colour (old palette slots 243-245).
/// Kept for a later migration milestone.
#[must_use]
#[allow(dead_code)]
pub fn is_dither_fill_color(color: Rgba) -> bool {
    color == BG_LEFT || color == BG_RIGHT || color == BG_ORDER
}

/// Fill palette slots 216..=255 with the standard UI palette colours.
/// Bridge function — only used by the legacy palette mutation path.
pub fn apply_standard_ui_palette(palette: &mut Palette) {
    for (i, &rgb) in STANDARD_UI_PALETTE.iter().enumerate() {
        palette.set(UI_PALETTE_BASE + i, rgb);
    }
}

pub const SUIT_PALETTE_BASE: usize = 215;
pub const SKI_PALETTE_INDEX: usize = 231;

// Jumper sprite source indices (what body/ski sprites natively contain)
pub const JUMPER_SUIT_SOURCE_SHADE_1: u8 = 216;
pub const JUMPER_SUIT_SOURCE_SHADE_3: u8 = 218;
pub const JUMPER_SKI_SOURCE: u8 = 231;

// Jumper private render slots (isolated from sprite/terrain/UI indices).
// These are within the standard UI palette range but no jump/replay view
// element uses them as color indices; terrain uses 0..=215; snow uses 232..=235.
// They are reserved — do not add non-jumper fillbox/text colors in this range
// to jump or replay views.
#[allow(dead_code)]
pub const JUMPER_SUIT_RENDER_SHADE_1: u8 = 219;
#[allow(dead_code)]
pub const JUMPER_SUIT_RENDER_SHADE_3: u8 = 222;
#[allow(dead_code)]
pub const JUMPER_SKI_RENDER: u8 = 230;

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

pub fn ski_rgb(col: usize) -> [u8; 3] {
    let col = col.min(SKI_COLORS.len() - 1);
    SKI_COLORS[col]
}

/// Return an arbitrary shade (0..4) of a suit colour as an Rgba.
#[must_use]
pub fn suit_color_shade(col: usize, shade: usize) -> Rgba {
    let rgb = suit_shade_rgba(col)[shade];
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

/// Return the ski colour as an Rgba.
#[must_use]
pub fn ski_color(col: usize) -> Rgba {
    let rgb = ski_rgb(col);
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

pub fn apply_suit_palette_at(palette: &mut Palette, col: usize, target_base: usize) {
    let colors = suit_shade_rgba(col);
    for (i, rgb) in colors.iter().enumerate() {
        palette.set(target_base + i, *rgb);
    }
}

pub fn apply_suit_palette(palette: &mut Palette, col: usize) {
    apply_suit_palette_at(palette, col, SUIT_PALETTE_BASE);
}

pub fn apply_ski_palette_at(palette: &mut Palette, col: usize, target: usize) {
    palette.set(target, ski_rgb(col));
}

pub fn apply_ski_palette(palette: &mut Palette, col: usize) {
    apply_ski_palette_at(palette, col, SKI_PALETTE_INDEX);
}

#[allow(dead_code)]
pub fn apply_jumper_palette(palette: &mut Palette, suit_color: usize, ski_color: usize) {
    let suit = suit_shade_rgba(suit_color);
    palette.set(JUMPER_SUIT_RENDER_SHADE_1 as usize, suit[1]);
    palette.set(JUMPER_SUIT_RENDER_SHADE_3 as usize, suit[3]);
    palette.set(JUMPER_SKI_RENDER as usize, ski_rgb(ski_color));
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
    const REPLACE_LOGO: [[u8; 3]; 4] = [[46, 46, 63], [32, 32, 63], [51, 51, 51], [38, 38, 38]];
    let col = col.min(3);
    palette.set(253, REPLACE_LOGO[col * 2]);
    palette.set(254, REPLACE_LOGO[col * 2 + 1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jumper_palette_sets_private_render_slots() {
        let mut pal = Palette::new();
        // Pre-set source slots to known values so we can detect tampering
        for i in 0..=255 {
            pal.set(i, [99, 99, 99]);
        }

        apply_jumper_palette(&mut pal, 1, 2);

        // Should use private slots 219, 222, 230
        assert_ne!(
            pal.color(JUMPER_SUIT_RENDER_SHADE_1 as usize),
            [99, 99, 99],
            "render shade 1 should be set"
        );
        assert_ne!(
            pal.color(JUMPER_SUIT_RENDER_SHADE_3 as usize),
            [99, 99, 99],
            "render shade 3 should be set"
        );
        assert_ne!(
            pal.color(JUMPER_SKI_RENDER as usize),
            [99, 99, 99],
            "ski render should be set"
        );
    }

    #[test]
    fn jumper_palette_does_not_touch_source_slots() {
        let mut pal = Palette::new();
        // Set source slots to a sentinel value
        for s in [
            JUMPER_SUIT_SOURCE_SHADE_1,
            JUMPER_SUIT_SOURCE_SHADE_3,
            JUMPER_SKI_SOURCE,
        ] {
            pal.set(s as usize, [42, 42, 42]);
        }

        apply_jumper_palette(&mut pal, 1, 2);

        // Source slots should remain unchanged
        assert_eq!(
            pal.color(JUMPER_SUIT_SOURCE_SHADE_1 as usize),
            [42, 42, 42],
            "source shade 1 unchanged"
        );
        assert_eq!(
            pal.color(JUMPER_SUIT_SOURCE_SHADE_3 as usize),
            [42, 42, 42],
            "source shade 3 unchanged"
        );
        assert_eq!(
            pal.color(JUMPER_SKI_SOURCE as usize),
            [42, 42, 42],
            "source ski unchanged"
        );
    }
}
