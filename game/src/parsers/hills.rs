use crate::data::records::{HillCatalog, HillInfo};
use crate::parsers::{AssetParser, ParseError};

pub struct HillBaseParser;

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| b != b' ' && b != b'\r')
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|&b| b != b' ' && b != b'\r')
        .map(|p| p + 1)
        .unwrap_or(0);
    &bytes[start..end]
}

fn decode_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(trim_ascii(bytes)).to_string()
}

impl AssetParser<HillCatalog> for HillBaseParser {
    fn parse(data: &[u8]) -> Result<HillCatalog, ParseError> {
        let lines: Vec<String> = data.split(|&b| b == b'\n').map(decode_line).collect();
        let mut hills = Vec::new();
        let mut idx = 0;

        while idx < lines.len() {
            if !lines[idx].starts_with('*') {
                idx += 1;
                continue;
            }
            if idx + 2 >= lines.len() {
                break;
            }
            let name = lines[idx + 1].clone();
            let kr = lines[idx + 2].parse::<i64>().unwrap_or(0);
            hills.push(HillInfo { name, kr });
            idx += 3;
        }

        if hills.is_empty() {
            return Err(ParseError {
                message: "HILLBASE.SKI has no hills".to_string(),
                byte_offset: None,
            });
        }

        Ok(HillCatalog::new(hills))
    }

    fn validate(data: &[u8]) -> bool {
        data.starts_with(b"-HILLBASE.SKI-")
    }
}
