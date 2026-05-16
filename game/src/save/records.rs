use crate::data::records::RecordStore;
use crate::save::crypt::crypt;
use crate::utils::pascal_encode;

const NUM_TOPS: usize = 41;
const NUM_HILL_RECORDS: usize = 20;

fn write_line(out: &mut Vec<u8>, text: &str) {
    out.extend(&pascal_encode(text));
    out.push(b'\n');
}

fn write_crypt(out: &mut Vec<u8>, value: i64, order: usize) {
    out.extend(&crypt(value, order));
    out.push(b'\n');
}

pub fn records_to_bytes(store: &RecordStore) -> Vec<u8> {
    let mut out = Vec::new();

    write_line(&mut out, "HISCORE.SKI - !!! DO NOT ATTEMPT TO EDIT THIS FILE !!!");

    for order in 1..=NUM_TOPS {
        if let Some(top) = store.top(order) {
            write_line(&mut out, &top.name);
            write_crypt(&mut out, top.pos as i64, order);
            write_crypt(&mut out, top.score, order);
        } else {
            write_line(&mut out, "");
            write_crypt(&mut out, 0, order);
            write_crypt(&mut out, 0, order);
        }
    }

    for order in 1..=NUM_HILL_RECORDS {
        if let Some(record) = store.hill_record(order) {
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

    for order in 1..=NUM_TOPS {
        if let Some(top) = store.top(order) {
            write_line(&mut out, &top.time);
        } else {
            write_line(&mut out, "");
        }
    }

    for order in 1..=NUM_HILL_RECORDS {
        if let Some(record) = store.hill_record(order) {
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
    use crate::parsers::records::RecordsParser;
    use crate::parsers::AssetParser;

    #[test]
    fn roundtrip_preserves_data() {
        let original = include_bytes!("../../assets/HISCORE.SKI");
        let store = RecordsParser::parse(original).expect("parse HISCORE.SKI");
        let rewritten = records_to_bytes(&store);
        let reparsed = RecordsParser::parse(&rewritten).expect("re-parse rewritten HISCORE.SKI");

        assert_eq!(store.top(1).map(|t| t.name.as_str()), reparsed.top(1).map(|t| t.name.as_str()));
        assert_eq!(store.top(1).map(|t| t.score), reparsed.top(1).map(|t| t.score));
        assert_eq!(store.hill_record(1).map(|r| r.name.as_str()), reparsed.hill_record(1).map(|r| r.name.as_str()));
        assert_eq!(store.hill_record(1).map(|r| r.len), reparsed.hill_record(1).map(|r| r.len));

        for i in 1..=20 {
            assert_eq!(store.top(i).map(|t| t.name.as_str()), reparsed.top(i).map(|t| t.name.as_str()));
            assert_eq!(store.top(i).map(|t| t.score), reparsed.top(i).map(|t| t.score));
        }
    }
}
