use crate::ui::paint::PaintCtx;
use crate::ui::widget::{Widget, Props, WidgetId};

#[derive(Debug, Clone)]
pub struct FillBoxProps {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: u8,
}

impl Props for FillBoxProps {}

#[derive(Debug)]
pub struct FillBox(pub FillBoxProps);

impl FillBox {
    pub fn new(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self(FillBoxProps { x, y, w, h, color })
    }
}

impl Widget for FillBox {
    fn paint(&self, ctx: &mut PaintCtx, _id: WidgetId) {
        ctx.fill_rect(self.0.x, self.0.y, self.0.w, self.0.h, self.0.color);
    }
}

pub struct BoxProps {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: u8,
}

impl Props for BoxProps {}

pub struct DrawBox(pub BoxProps);

impl DrawBox {
    pub fn new(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self(BoxProps { x, y, w, h, color })
    }

    fn draw_line(&self, ctx: &mut PaintCtx, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx as i32 - dy as i32;
        let mut x = x1;
        let mut y = y1;
        loop {
            ctx.set_pixel(x, y, self.0.color);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy as i32 {
                err -= dy as i32;
                x += sx;
            }
            if e2 < dx as i32 {
                err += dx as i32;
                y += sy;
            }
        }
    }

    fn draw_h_line(&self, ctx: &mut PaintCtx, x1: i32, x2: i32, y: i32) {
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        for x in min_x..=max_x {
            ctx.set_pixel(x, y, self.0.color);
        }
    }

    fn draw_v_line(&self, ctx: &mut PaintCtx, y1: i32, y2: i32, x: i32) {
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        for y in min_y..=max_y {
            ctx.set_pixel(x, y, self.0.color);
        }
    }
}

impl Widget for DrawBox {
    fn paint(&self, ctx: &mut PaintCtx, _id: WidgetId) {
        let x2 = self.0.x + self.0.w - 1;
        let y2 = self.0.y + self.0.h - 1;
        self.draw_h_line(ctx, self.0.x, x2, self.0.y);
        self.draw_h_line(ctx, self.0.x, x2, y2);
        self.draw_v_line(ctx, self.0.y, y2, self.0.x);
        self.draw_v_line(ctx, self.0.y, y2, x2);
    }
}