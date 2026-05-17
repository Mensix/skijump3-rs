use crate::parsers::{AssetParser, ParseError};
use crate::text::layout;
use std::cell::Cell;

const NUM_STR: usize = 599;

#[derive(Debug, Clone)]
pub struct LangBase {
    all_strings: Vec<Vec<String>>,
    pub languages: Vec<String>,
    pub selected: Cell<usize>,
}

impl LangBase {
    #[must_use]
    pub fn lstr(&self, index: usize) -> &str {
        let lang = self.selected.get();
        if lang < self.all_strings.len() && index < self.all_strings[lang].len() {
            &self.all_strings[lang][index]
        } else {
            "?"
        }
    }
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
        let trimmed = layout::trim_ascii(line);
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

impl AssetParser for LangBaseParser {
    type Output = LangBase;
    fn parse(data: &[u8]) -> Result<LangBase, ParseError> {
        let languages = parse_language_names(data);
        let num_languages = languages.len();

        let mut all_strings: Vec<Vec<String>> = (0..num_languages)
            .map(|_| (0..=NUM_STR).map(|_| "?".to_string()).collect())
            .collect();
        let mut current_lang: Option<usize> = None;

        for line in data.split(|&b| b == b'\n') {
            let trimmed = layout::trim_ascii(line);

            if trimmed.is_empty() {
                continue;
            }

            if trimmed[0] == b'*' {
                current_lang = None;
                if trimmed.len() > 1 {
                    let letter = trimmed[1];
                    if letter.is_ascii_uppercase() {
                        let idx = (letter - b'A') as usize;
                        if idx < num_languages {
                            current_lang = Some(idx);
                        }
                    }
                }
                continue;
            }

            if trimmed[0] == b'/' {
                continue;
            }

            if let Some(lang_idx) = current_lang {
                if let Some(colon) = trimmed.iter().position(|&b| b == b':') {
                    let num_str = &trimmed[..colon];
                    let val = &trimmed[colon + 1..];
                    if let Some(index) = parse_num(num_str) {
                        if index <= NUM_STR {
                            all_strings[lang_idx][index] = decode_val(val);
                        }
                    }
                }
            }
        }

        Ok(LangBase {
            all_strings,
            languages,
            selected: Cell::new(0),
        })
    }
}
