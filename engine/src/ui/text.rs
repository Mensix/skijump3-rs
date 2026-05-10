use crate::ui::paint::PaintCtx;
use crate::ui::widget::{Widget, Props, WidgetId};

#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: String,
    pub x: i32,
    pub y: i32,
    pub color: u8,
    pub scale: i32,
}

impl Props for TextProps {}

#[derive(Debug)]
pub struct Text {
    pub props: TextProps,
}

impl Text {
    pub fn new(content: &str, x: i32, y: i32, color: u8, scale: i32) -> Self {
        Self { props: TextProps { content: content.to_string(), x, y, color, scale } }
    }
}

impl Widget for Text {
    fn paint(&self, ctx: &mut PaintCtx, _id: WidgetId) {
        let mut px = self.props.x;
        for byte in self.props.content.bytes() {
            let char_idx = Self::char_to_sprite(byte);
            ctx.draw_glyph(char_idx, px, self.props.y, self.props.color, self.props.scale);
            px += 8 * self.props.scale;
        }
    }
}

impl Text {
    fn char_to_sprite(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A' + 1,
            b'a'..=b'z' => c - b'a' + 27,
            b'0'..=b'9' => c - b'0' + 53,
            b' ' => 0,
            b'.' => 63,
            b',' => 64,
            b'!' => 65,
            b'?' => 66,
            _ => 0,
        }
    }
}