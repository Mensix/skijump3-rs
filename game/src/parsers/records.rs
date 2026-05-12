use crate::data::records::{HillRecord, Hiscore, RecordStore};
use crate::parsers::{AssetParser, ParseError};

const NUM_TOPS: usize = 41;
const NUM_HILL_RECORDS: usize = 20;

pub struct RecordsParser;

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

fn uncrypt(input: &str, order: usize) -> i64 {
    let mut bytes = input.as_bytes().to_vec();
    if bytes.len() < 8 {
        return 0;
    }

    let high = bytes[4];
    bytes.remove(4);
    let chk1 = bytes[1];
    bytes.remove(1);
    let chk2 = bytes[1];
    bytes.remove(1);

    if bytes.len() < 5 {
        return 0;
    }

    let mut digits = String::with_capacity(5);
    for idx in (0..5).rev() {
        digits.push((bytes[idx].saturating_sub(21)) as char);
    }

    let mut value = digits.parse::<i64>().unwrap_or(0);
    if high > 74 {
        value += 100_000 * i64::from(high - 75);
    }

    let expected_value = 68 + 2 * (((value % 7) as u8) ^ 1);
    let expected_order = 65 + (((order as u8) ^ 33) % 19);
    if chk1 != expected_value || chk2 != expected_order {
        return 0;
    }

    value
}

impl AssetParser<RecordStore> for RecordsParser {
    fn parse(data: &[u8]) -> Result<RecordStore, ParseError> {
        let lines: Vec<String> = data.split(|&b| b == b'\n').map(decode_line).collect();
        let min_lines = 1 + NUM_TOPS * 3 + NUM_HILL_RECORDS * 2 + 3 + NUM_TOPS + NUM_HILL_RECORDS;
        if lines.len() < min_lines {
            return Err(ParseError {
                message: "HISCORE.SKI is too short".to_string(),
                byte_offset: None,
            });
        }

        let mut cursor = 1;
        let mut top = Vec::with_capacity(NUM_TOPS);
        for order in 1..=NUM_TOPS {
            let name = lines[cursor].clone();
            let pos = uncrypt(&lines[cursor + 1], order) as usize;
            let score = uncrypt(&lines[cursor + 2], order);
            top.push(Hiscore {
                name,
                pos,
                score,
                time: String::new(),
            });
            cursor += 3;
        }

        let mut hill_records = Vec::with_capacity(NUM_HILL_RECORDS);
        for order in 1..=NUM_HILL_RECORDS {
            let name = lines[cursor].clone();
            let len = uncrypt(&lines[cursor + 1], order);
            hill_records.push(HillRecord {
                name,
                len,
                time: String::new(),
            });
            cursor += 2;
        }

        cursor += 3;
        for record in &mut top {
            record.time = lines[cursor].clone();
            cursor += 1;
        }
        for record in &mut hill_records {
            record.time = lines[cursor].clone();
            cursor += 1;
        }

        Ok(RecordStore::new(top, hill_records))
    }

    fn validate(data: &[u8]) -> bool {
        data.starts_with(b"HISCORE.SKI")
    }
}
