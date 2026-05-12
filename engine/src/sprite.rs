#[derive(Debug, Clone)]
pub struct SpriteData {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

impl SpriteData {
    pub fn blit_to(&self, pixels: &mut [u8], screen_w: u32, dst_x: i32, dst_y: i32) {
        let start_x = dst_x - (self.width as i32 / 2);
        let start_y = dst_y - (self.height as i32 / 2);
        for yy in 0..self.height as i32 {
            for xx in 0..self.width as i32 {
                let src_idx = (yy * self.width as i32 + xx) as usize;
                if src_idx < self.data.len() {
                    let px = start_x + xx;
                    let py = start_y + yy;
                    if px >= 0 && py >= 0 && (px as u32) < screen_w && py < 200 {
                        let idx = (py as usize) * (screen_w as usize) + (px as usize);
                        pixels[idx] = self.data[src_idx];
                    }
                }
            }
        }
    }
}
