use engine::color::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextTheme {
    pub body: Rgba,
    pub gold: Rgba,
    pub teal: Rgba,
    pub gray: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundTheme {
    pub dark: Rgba,
    pub purple: Rgba,
    pub red: Rgba,
    pub red_bright: Rgba,
    pub green: Rgba,
    pub team: Rgba,
    pub worldcup: Rgba,
    pub darkest: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillTheme {
    pub purple: Rgba,
    pub gold: Rgba,
    pub teal: Rgba,
    pub dark: Rgba,
    pub gray: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialTheme {
    pub start_dq_lit: Rgba,
    pub start_dq_dark: Rgba,
    pub start_ready_lit: Rgba,
    pub start_ready_dark: Rgba,
    pub replay_active: Rgba,
    pub replay_inactive: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub text: TextTheme,
    pub background: BackgroundTheme,
    pub fill: FillTheme,
    pub material: MaterialTheme,
    pub black: Rgba,
}

impl Theme {
    pub const DEFAULT: Self = Self {
        text: TextTheme {
            body: Rgba::from_rgb6(63, 63, 63),
            gold: Rgba::from_rgb6(63, 57, 9),
            teal: Rgba::from_rgb6(9, 57, 63),
            gray: Rgba::from_rgb6(44, 44, 44),
        },
        background: BackgroundTheme {
            dark: Rgba::from_rgb6(5, 8, 20),
            purple: Rgba::from_rgb6(18, 13, 34),
            red: Rgba::from_rgb6(34, 13, 18),
            red_bright: Rgba::from_rgb6(43, 16, 23),
            green: Rgba::from_rgb6(0, 25, 0),
            team: Rgba::from_rgb6(28, 8, 24),
            worldcup: Rgba::from_rgb6(47, 0, 0),
            darkest: Rgba::from_rgb6(10, 10, 10),
        },
        fill: FillTheme {
            purple: Rgba::from_rgb6(23, 16, 43),
            gold: Rgba::from_rgb6(52, 47, 0),
            teal: Rgba::from_rgb6(0, 47, 52),
            dark: Rgba::from_rgb6(5, 8, 22),
            gray: Rgba::from_rgb6(20, 20, 20),
        },
        material: MaterialTheme {
            start_dq_lit: Rgba::from_rgb6(54, 10, 10),
            start_dq_dark: Rgba::from_rgb6(47, 0, 0),
            start_ready_lit: Rgba::from_rgb6(10, 54, 10),
            start_ready_dark: Rgba::from_rgb6(0, 47, 0),
            replay_active: Rgba::from_rgb6(10, 63, 20),
            replay_inactive: Rgba::rgb(0, 0, 0),
        },
        black: Rgba::rgb(0, 0, 0),
    };
}

pub const THEME: Theme = Theme::DEFAULT;

pub const FONT_BODY: Rgba = THEME.text.body;
pub const FONT_GOLD: Rgba = THEME.text.gold;
pub const FONT_TEAL: Rgba = THEME.text.teal;
pub const FONT_GRAY: Rgba = THEME.text.gray;
pub const BG_DARK: Rgba = THEME.background.dark;
pub const BG_PURPLE: Rgba = THEME.background.purple;
pub const BG_RED: Rgba = THEME.background.red;
pub const BG_RED_BRIGHT: Rgba = THEME.background.red_bright;
pub const BG_GREEN: Rgba = THEME.background.green;
pub const BG_TEAM: Rgba = THEME.background.team;
pub const BG_WORLDCUP: Rgba = THEME.background.worldcup;
pub const BG_DARKEST: Rgba = THEME.background.darkest;
pub const FILL_PURPLE: Rgba = THEME.fill.purple;
pub const FILL_GOLD: Rgba = THEME.fill.gold;
pub const FILL_TEAL: Rgba = THEME.fill.teal;
pub const FILL_DARK: Rgba = THEME.fill.dark;
pub const FILL_GRAY: Rgba = THEME.fill.gray;
pub const BLACK: Rgba = THEME.black;
