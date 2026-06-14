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
    pub default: Rgba,
    pub header: Rgba,
    pub gold: Rgba,
    pub greet: Rgba,
    pub name: Rgba,
    pub new: Rgba,
    pub back: Rgba,
    pub help: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundTheme {
    pub erase: Rgba,
    pub list: Rgba,
    pub left: Rgba,
    pub right: Rgba,
    pub order: Rgba,
    pub right_bright: Rgba,
    pub koth: Rgba,
    pub team_cup: Rgba,
    pub world_cup: Rgba,
    pub four_hills: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillTheme {
    pub border: Rgba,
    pub highlight: Rgba,
    pub turquoise: Rgba,
    pub line: Rgba,
    pub dim: Rgba,
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
            default: Rgb6(63, 63, 63).rgba(),
            header: Rgb6(63, 57, 9).rgba(),
            gold: Rgb6(63, 57, 9).rgba(),
            greet: Rgb6(9, 57, 63).rgba(),
            name: Rgb6(63, 63, 63).rgba(),
            new: Rgb6(63, 57, 9).rgba(),
            back: Rgb6(63, 63, 63).rgba(),
            help: Rgb6(44, 44, 44).rgba(),
        },
        background: BackgroundTheme {
            erase: Rgb6(5, 8, 20).rgba(),
            list: Rgb6(5, 8, 20).rgba(),
            left: Rgb6(18, 13, 34).rgba(),
            right: Rgb6(34, 13, 18).rgba(),
            order: Rgb6(18, 13, 34).rgba(),
            right_bright: Rgb6(43, 16, 23).rgba(),
            koth: Rgb6(0, 25, 0).rgba(),
            team_cup: Rgb6(28, 8, 24).rgba(),
            world_cup: Rgb6(47, 0, 0).rgba(),
            four_hills: Rgb6(10, 10, 10).rgba(),
        },
        fill: FillTheme {
            border: Rgb6(23, 16, 43).rgba(),
            highlight: Rgb6(52, 47, 0).rgba(),
            turquoise: Rgb6(0, 47, 52).rgba(),
            line: Rgb6(5, 8, 22).rgba(),
            dim: Rgb6(20, 20, 20).rgba(),
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

pub const FONT_DEFAULT: Rgba = THEME.text.default;
pub const FONT_HEADER: Rgba = THEME.text.header;
pub const FONT_GOLD: Rgba = THEME.text.gold;
pub const FONT_GREET: Rgba = THEME.text.greet;
pub const FONT_NAME: Rgba = THEME.text.name;
pub const FONT_NEW: Rgba = THEME.text.new;
pub const FONT_BACK: Rgba = THEME.text.back;
pub const FONT_HELP: Rgba = THEME.text.help;
pub const BG_ERASE: Rgba = THEME.background.erase;
pub const BG_LIST: Rgba = THEME.background.list;
pub const BG_LEFT: Rgba = THEME.background.left;
pub const BG_RIGHT: Rgba = THEME.background.right;
pub const BG_ORDER: Rgba = THEME.background.order;
pub const BG_RIGHT_BRIGHT: Rgba = THEME.background.right_bright;
pub const BG_KOTH: Rgba = THEME.background.koth;
pub const BG_TEAMCUP: Rgba = THEME.background.team_cup;
pub const BG_WC: Rgba = THEME.background.world_cup;
pub const BG_4HILLS: Rgba = THEME.background.four_hills;
pub const FILL_BORDER: Rgba = THEME.fill.border;
pub const FILL_HIGHLIGHT: Rgba = THEME.fill.highlight;
pub const FILL_TURQUOISE: Rgba = THEME.fill.turquoise;
pub const FILL_LINE: Rgba = THEME.fill.line;
pub const FILL_DIM: Rgba = THEME.fill.dim;
pub const BLACK: Rgba = THEME.black;
