use engine::color::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb6(u8, u8, u8);

impl Rgb6 {
    const BLACK: Self = Self(0, 0, 0);

    const fn rgba(self) -> Rgba {
        Rgba::from_rgb6(self.0, self.1, self.2)
    }
}

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
            body: Rgb6(63, 63, 63).rgba(),
            gold: Rgb6(63, 57, 9).rgba(),
            teal: Rgb6(9, 57, 63).rgba(),
            gray: Rgb6(44, 44, 44).rgba(),
        },
        background: BackgroundTheme {
            dark: Rgb6(5, 8, 20).rgba(),
            purple: Rgb6(18, 13, 34).rgba(),
            red: Rgb6(34, 13, 18).rgba(),
            red_bright: Rgb6(43, 16, 23).rgba(),
            green: Rgb6(0, 25, 0).rgba(),
            team: Rgb6(28, 8, 24).rgba(),
            worldcup: Rgb6(47, 0, 0).rgba(),
            darkest: Rgb6(10, 10, 10).rgba(),
        },
        fill: FillTheme {
            purple: Rgb6(23, 16, 43).rgba(),
            gold: Rgb6(52, 47, 0).rgba(),
            teal: Rgb6(0, 47, 52).rgba(),
            dark: Rgb6(5, 8, 22).rgba(),
            gray: Rgb6(20, 20, 20).rgba(),
        },
        material: MaterialTheme {
            start_dq_lit: Rgb6(54, 10, 10).rgba(),
            start_dq_dark: Rgb6(47, 0, 0).rgba(),
            start_ready_lit: Rgb6(10, 54, 10).rgba(),
            start_ready_dark: Rgb6(0, 47, 0).rgba(),
            replay_active: Rgb6(10, 63, 20).rgba(),
            replay_inactive: Rgb6::BLACK.rgba(),
        },
        black: Rgb6::BLACK.rgba(),
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
