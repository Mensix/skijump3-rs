use crate::parsers::{AssetParser, ParseError};

const NUM_STR: usize = 599;

#[derive(Clone)]
pub struct LangBase {
    strings: Vec<String>,
    pub languages: Vec<String>,
}

impl LangBase {
    pub fn lstr(&self, index: usize) -> &str {
        if index < self.strings.len() {
            &self.strings[index]
        } else {
            "?"
        }
    }
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|&b| b != b' ' && b != b'\r').unwrap_or(bytes.len());
    let end = bytes.iter().rposition(|&b| b != b' ' && b != b'\r').map(|p| p + 1).unwrap_or(0);
    &bytes[start..end]
}

fn parse_num(bytes: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(bytes).ok()?;
    s.parse().ok()
}

fn decode_val(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn parse_language_names(data: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut expect_name = false;
    for line in data.split(|&b| b == b'\n') {
        let trimmed = trim_ascii(line);
        if trimmed.is_empty() {
            expect_name = false;
            continue;
        }
        if trimmed[0] == b'*' {
            expect_name = true;
            continue;
        }
        if expect_name {
            names.push(decode_val(trimmed));
            expect_name = false;
        }
    }
    names
}

pub struct LangBaseParser;

impl AssetParser<LangBase> for LangBaseParser {
    fn parse(data: &[u8]) -> Result<LangBase, ParseError> {
        let languages = parse_language_names(data);

        let mut strings: Vec<String> = (0..=NUM_STR).map(|_| "?".to_string()).collect();
        let mut in_english = false;

        for line in data.split(|&b| b == b'\n') {
            let trimmed = trim_ascii(line);

            if trimmed.is_empty() {
                continue;
            }

            if trimmed[0] == b'*' {
                if in_english {
                    break;
                }
                if trimmed.len() > 1 && trimmed[1] == b'A' {
                    in_english = true;
                }
                continue;
            }

            if !in_english || trimmed[0] == b'/' {
                continue;
            }

            if let Some(colon) = trimmed.iter().position(|&b| b == b':') {
                let num_str = &trimmed[..colon];
                let val = &trimmed[colon + 1..];
                if let Some(index) = parse_num(num_str) {
                    if index <= NUM_STR {
                        strings[index] = decode_val(val);
                    }
                }
            }
        }

        Ok(LangBase { strings, languages })
    }

    fn validate(data: &[u8]) -> bool {
        data.len() > 10
    }
}
