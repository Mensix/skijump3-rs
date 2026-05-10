use crate::loaders::assets::AssetStore;
use crate::parsers::{AssetParser, pcx::{DecodedPcx, PcxParser}};
use engine::palette::Palette;

pub const MAIN_PCX: &str = "MAIN.PCX";

pub struct MainMenu {
    pub pixels: Vec<u8>,
    pub palette: Palette,
}

impl MainMenu {
    pub fn load() -> Result<Self, String> {
        let data = AssetStore::read(MAIN_PCX).map_err(|e| e.to_string())?;

        let decoded: DecodedPcx = PcxParser::parse(&data).map_err(|e| e.to_string())?;

        Ok(Self {
            pixels: decoded.pixels,
            palette: decoded.palette,
        })
    }
}