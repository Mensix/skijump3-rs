use crate::parsers::{AssetParser, ParseError};
use crate::text::encoding;
use crate::text::layout;

pub struct NamesParser;

impl AssetParser<Vec<String>> for NamesParser {
    fn parse(data: &[u8]) -> Result<Vec<String>, ParseError> {
        let mut names: Vec<String> = Vec::new();
        let mut in_names = false;

        for line_bytes in data.split(|&b| b == b'\n') {
            let trimmed = layout::trim_ascii(line_bytes);

            if trimmed.is_empty() {
                continue;
            }

            if trimmed[0] == b'*' {
                if trimmed.len() >= 3
                    && &trimmed[..3] == b"***"
                    && trimmed.windows(5).any(|w| w == b"TEAMS")
                {
                    break;
                }
                in_names = true;
                continue;
            }

            if !in_names {
                continue;
            }

            if trimmed
                .iter()
                .all(|&b| b.is_ascii_digit() || b == b'.' || b == b' ')
            {
                continue;
            }

            let name = encoding::decode(trimmed).trim().to_string();
            if !name.is_empty() {
                names.push(name);
            }
        }

        Ok(names)
    }
}
