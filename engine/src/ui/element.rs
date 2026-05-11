use crate::ui::paint::PaintCtx;
use crate::ui::Font;

#[derive(Clone, Debug)]
pub enum Cmd {
    None,
    Quit,
    Navigate(RouteTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    OptionsMenu,
    Profiles,
    Play(u8),
    Results { score: u32, hill: u8 },
}

impl RouteTarget {
    pub fn as_cmd(self) -> Cmd {
        Cmd::Navigate(self)
    }
}

pub enum Element {
    Image(Vec<u8>, u32, u32),
    Text { text: String, x: i32, y: i32, color: u8 },
    Sprite(u8, i32, i32),
    Fillbox { x: i32, y: i32, w: i32, h: i32, color: u8 },
    Box { x: i32, y: i32, w: i32, h: i32, color: u8 },
    Button {
        text: String,
        x: i32,
        y: i32,
        selected: bool,
        cmd: Cmd,
    },
    Container(Vec<Element>),
}

impl Element {
    pub fn render(&self, ctx: &mut PaintCtx, font: &Font, sprites: &[Vec<u8>]) {
        match self {
            Element::Image(pixels, w, h) => {
                debug_assert!(
                    pixels.len() >= (*w as usize) * (*h as usize),
                    "Image pixels too small: expected {}x{}, got {} bytes",
                    w, h, pixels.len()
                );
                let dst_w = ctx.width.min(*w);
                let dst_h = ctx.height.min(*h);
                for y in 0..dst_h {
                    let src_row = (y as usize) * (*w as usize);
                    let dst_row = (y as usize) * (ctx.width as usize);
                    let src = &pixels[src_row..src_row + dst_w as usize];
                    ctx.pixels[dst_row..dst_row + dst_w as usize].copy_from_slice(src);
                }
            }
            Element::Text { text, x, y, color } => {
                font.blit_string_color(ctx.pixels, ctx.width, text, *x, *y, *color);
            }
            Element::Fillbox { x, y, w, h, color } => {
                ctx.fill_rect(*x, *y, *w, *h, *color);
            }
            Element::Box { x, y, w, h, color } => {
                ctx.fill_rect(*x, *y, *w, 1, *color);
                ctx.fill_rect(*x, *y + *h - 1, *w, 1, *color);
                ctx.fill_rect(*x, *y, 1, *h, *color);
                ctx.fill_rect(*x + *w - 1, *y, 1, *h, *color);
            }
            Element::Button { text, x, y, selected, .. } => {
                if *selected {
                    ctx.fill_rect(*x - 10, *y - 1, 120, 10, 246);
                }
                font.blit_string(ctx.pixels, ctx.width, text, *x, *y);
            }
            Element::Container(children) => {
                for child in children {
                    child.render(ctx, font, sprites);
                }
            }
            Element::Sprite(_, _, _) => {}
        }
    }

    pub fn text(text: impl Into<String>, x: i32, y: i32) -> Self {
        Self::Text { text: text.into(), x, y, color: 15 }
    }

    pub fn text_color(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self::Text { text: text.into(), x, y, color }
    }

    pub fn sprite(idx: u8, x: i32, y: i32) -> Self {
        Self::Sprite(idx, x, y)
    }

    pub fn image(pixels: Vec<u8>, w: u32, h: u32) -> Self {
        Self::Image(pixels, w, h)
    }

    pub fn fillbox(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Fillbox { x, y, w, h, color }
    }

    pub fn box_(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Box { x, y, w, h, color }
    }

    pub fn button(text: impl Into<String>, x: i32, y: i32, selected: bool, cmd: Cmd) -> Self {
        Self::Button { text: text.into(), x, y, selected, cmd }
    }

    pub fn container(children: Vec<Element>) -> Self {
        Self::Container(children)
    }
}