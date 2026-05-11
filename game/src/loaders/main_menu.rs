use crate::loaders::assets::AssetStore;
use crate::parsers::{AssetParser, anim::AnimParser, pcx::{DecodedPcx, PcxParser}};
use engine::palette::Palette;
use engine::ui::Font;

pub const MAIN_PCX: &str = "MAIN.PCX";
pub const ANIM_SKI: &str = "ANIM.SKI";

pub struct MainMenuAssets {
    pub background: DecodedPcx,
    pub font: Font,
}

impl MainMenuAssets {
    pub fn load() -> Result<Self, String> {
        let pcx_data = AssetStore::read(MAIN_PCX).map_err(|e| e.to_string())?;
        let decoded: DecodedPcx = PcxParser::parse(&pcx_data).map_err(|e| e.to_string())?;

        let anim_data = AssetStore::read(ANIM_SKI).map_err(|e| e.to_string())?;
        let sprites = AnimParser::parse(&anim_data).map_err(|e| e.to_string())?;

        let mut font = Font::new();
        for (i, sprite) in sprites.iter().enumerate() {
            if i < 67 {
                font.set_glyph(i, sprite.data.clone(), sprite.width, sprite.height, sprite.center_x, sprite.center_y);
            }
        }

        Ok(Self { background: decoded, font })
    }

    pub fn compose(&self) -> (Vec<u8>, Palette) {
        let mut pixels = self.background.pixels.clone();
        let font = &self.font;

        let items = [
            ("WORLD CUP", 20),
            ("TEAM CUP", 35),
            ("4 HILLS", 50),
            ("KING OF THE HILL", 65),
            ("PRACTICE", 80),
            ("CUSTOM CUP", 95),
            ("REPLAYS", 110),
            ("PROFILES", 125),
            ("SETTINGS", 140),
            ("HILL RECORDS", 155),
            ("EXIT", 170),
        ];

        for (content, y) in items {
            font.blit_string(&mut pixels, 320, content, 20, y);
        }

        (pixels, self.background.palette.clone())
    }
}