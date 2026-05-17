use crate::data::records::RecordStore;
use crate::save::crypt::crypt;
use crate::text::encoding;

const NUM_TOPS: usize = 41;
const NUM_HILL_RECORDS: usize = 20;

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
    use crate::parsers::records::RecordsParser;
    use crate::parsers::AssetParser;

    #[test]
    fn roundtrip_preserves_data() {
        let original = include_bytes!("../../assets/HISCORE.SKI");
        let store = RecordsParser::parse(original).expect("parse HISCORE.SKI");
        let rewritten = records_to_bytes(&store);
        let reparsed = RecordsParser::parse(&rewritten).expect("re-parse rewritten HISCORE.SKI");

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
