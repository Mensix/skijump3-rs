use crate::data::records::{HillRecord, Hiscore, RecordStore};
use crate::save::crypt::crypt;
use crate::text::encoding;
use crate::text::layout;

const NUM_TOPS: usize = 41;
const NUM_HILL_RECORDS: usize = 20;

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

fn decode_line(bytes: &[u8]) -> String {
    encoding::decode(layout::trim_ascii(bytes))
}

impl RecordStore {
    pub fn from_hiscore_bytes(data: &[u8]) -> Result<Self, String> {
        let lines: Vec<String> = data.split(|&b| b == b'\n').map(decode_line).collect();
        let min_lines = 1 + NUM_TOPS * 3 + NUM_HILL_RECORDS * 2 + 3 + NUM_TOPS + NUM_HILL_RECORDS;
        if lines.len() < min_lines {
            return Err("HISCORE.SKI is too short".to_string());
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

fn write_line(out: &mut Vec<u8>, text: &str) {
    out.extend(&encoding::encode(text));
    out.push(b'\n');
}

fn write_crypt(out: &mut Vec<u8>, value: i64, order: usize) {
    out.extend(&crypt(value, order));
    out.push(b'\n');
}

#[must_use]
pub fn records_to_bytes(store: &RecordStore) -> Vec<u8> {
    let mut out = Vec::new();

    write_line(
        &mut out,
        "HISCORE.SKI - !!! DO NOT ATTEMPT TO EDIT THIS FILE !!!",
    );

    for idx in 0..NUM_TOPS {
        let order = idx + 1; // Pascal file format uses 1-based crypt keys
        if let Some(top) = store.top(idx) {
            write_line(&mut out, &top.name);
            write_crypt(&mut out, top.pos as i64, order);
            write_crypt(&mut out, top.score, order);
        } else {
            write_line(&mut out, "");
            write_crypt(&mut out, 0, order);
            write_crypt(&mut out, 0, order);
        }
    }

    for idx in 0..NUM_HILL_RECORDS {
        let order = idx + 1;
        if let Some(record) = store.hill_record(idx) {
            write_line(&mut out, &record.name);
            write_crypt(&mut out, record.len, order);
        } else {
            write_line(&mut out, "");
            write_crypt(&mut out, 0, order);
        }
    }

    write_line(&mut out, "DO NOT ATTEMPT TO EDIT THIS FILE !!!");
    write_line(&mut out, "");
    write_line(&mut out, "");

    for idx in 0..NUM_TOPS {
        if let Some(top) = store.top(idx) {
            write_line(&mut out, &top.time);
        } else {
            write_line(&mut out, "");
        }
    }

    for idx in 0..NUM_HILL_RECORDS {
        if let Some(record) = store.hill_record(idx) {
            write_line(&mut out, &record.time);
        } else {
            write_line(&mut out, "");
        }
    }

    out
}

impl super::SaveFormat for RecordStore {
    fn to_bytes(&self) -> Vec<u8> {
        records_to_bytes(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_data() {
        let original = include_bytes!("../../assets/HISCORE.SKI");
        let store = RecordStore::from_hiscore_bytes(original).expect("parse HISCORE.SKI");
        let rewritten = records_to_bytes(&store);
        let reparsed = RecordStore::from_hiscore_bytes(&rewritten).expect("re-parse rewritten HISCORE.SKI");

        assert_eq!(
            store.top(0).map(|t| t.name.as_str()),
            reparsed.top(0).map(|t| t.name.as_str())
        );
        assert_eq!(
            store.top(0).map(|t| t.score),
            reparsed.top(0).map(|t| t.score)
        );
        assert_eq!(
            store.hill_record(0).map(|r| r.name.as_str()),
            reparsed.hill_record(0).map(|r| r.name.as_str())
        );
        assert_eq!(
            store.hill_record(0).map(|r| r.len),
            reparsed.hill_record(0).map(|r| r.len)
        );

        for i in 0..20 {
            assert_eq!(
                store.top(i).map(|t| t.name.as_str()),
                reparsed.top(i).map(|t| t.name.as_str())
            );
            assert_eq!(
                store.top(i).map(|t| t.score),
                reparsed.top(i).map(|t| t.score)
            );
        }
    }
}
