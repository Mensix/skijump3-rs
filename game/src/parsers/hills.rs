use crate::data::records::{HillCatalog, HillInfo};
use crate::parsers::{AssetParser, ParseError};
use crate::text::encoding;
use crate::text::layout;

pub struct HillBaseParser;

fn decode_line(bytes: &[u8]) -> String {
    encoding::decode(layout::trim_ascii(bytes))
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
            if idx + 12 >= lines.len() {
                break;
            }
            hills.push(HillInfo {
                name: lines[idx + 1].clone(),
                kr: parse_i64(&lines, idx + 2),
                front_index: lines[idx + 3].clone(),
                back_index: lines[idx + 4].clone(),
                back_brightness: parse_i64(&lines, idx + 5),
                back_mirror: parse_i64(&lines, idx + 6),
                vx_final: parse_i64(&lines, idx + 7),
                pk_hundred: parse_i64(&lines, idx + 8),
                pl_save_ten_thousand: parse_i64(&lines, idx + 9),
                author: lines[idx + 10].clone(),
                checksum: parse_i64(&lines, idx + 11),
                profile_checksum: parse_i64(&lines, idx + 12),
            });
            idx += 13;
        }

        if hills.is_empty() {
            return Err(ParseError {
                message: "HILLBASE.SKI has no hills".to_string(),
                byte_offset: None,
            });
        }

        Ok(HillCatalog::new(hills))
    }

}

fn parse_i64(lines: &[String], idx: usize) -> i64 {
    lines
        .get(idx)
        .and_then(|line| line.parse::<i64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_official_hill_metadata() {
        let catalog = HillBaseParser::parse(include_bytes!("../../assets/HILLBASE.SKI"))
            .expect("valid HILLBASE.SKI");
        let kuopio = catalog.hill(1).expect("first hill");

        assert_eq!(kuopio.name, "kuopio");
        assert_eq!(kuopio.kr, 120);
        assert_eq!(kuopio.front_index, "1");
        assert_eq!(kuopio.back_index, "0");
        assert_eq!(kuopio.back_brightness, 90);
        assert_eq!(kuopio.back_mirror, 0);
        assert_eq!(kuopio.vx_final, 148);
        assert_eq!(kuopio.pk_hundred, 89);
        assert_eq!(kuopio.pl_save_ten_thousand, 3217);
        assert_eq!(kuopio.author, "SJ v3.00 Original");
        assert_eq!(kuopio.checksum, 932638);
        assert_eq!(kuopio.profile_checksum, 1009542);
        assert_eq!(kuopio.pk(), 0.89);
        assert_eq!(kuopio.pl_save(), 0.3217);
    }
}
