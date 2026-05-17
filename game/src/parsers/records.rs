use crate::data::records::{HillRecord, Hiscore, RecordStore};
use crate::parsers::{AssetParser, ParseError};
use crate::text::encoding;
use crate::text::layout;

const NUM_TOPS: usize = 41;
const NUM_HILL_RECORDS: usize = 20;

pub struct RecordsParser;

fn decode_line(bytes: &[u8]) -> String {
    encoding::decode(layout::trim_ascii(bytes))
}

pub(crate) fn uncrypt(input: &str, order: usize) -> i64 {
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

        let mut top: Vec<Hiscore> = lines[1..]
            .chunks_exact(3)
            .take(NUM_TOPS)
            .enumerate()
            .map(|(order, chunk)| {
                let order = order + 1;
                Hiscore {
                    name: chunk[0].clone(),
                    pos: uncrypt(&chunk[1], order) as usize,
                    score: uncrypt(&chunk[2], order),
                    time: String::new(),
                }
            })
            .collect();

        let time_start = 1 + NUM_TOPS * 3;
        let mut hill_records: Vec<HillRecord> = lines[time_start..]
            .chunks_exact(2)
            .take(NUM_HILL_RECORDS)
            .enumerate()
            .map(|(order, chunk)| {
                let order = order + 1;
                HillRecord {
                    name: chunk[0].clone(),
                    len: uncrypt(&chunk[1], order),
                    time: String::new(),
                }
            })
            .collect();

        let time_offset = time_start + NUM_HILL_RECORDS * 2 + 3;
        for (i, record) in top.iter_mut().enumerate() {
            record.time = lines[time_offset + i].clone();
        }
        for (i, record) in hill_records.iter_mut().enumerate() {
            record.time = lines[time_offset + NUM_TOPS + i].clone();
        }

        Ok(RecordStore::new(top, hill_records))
    }

}
