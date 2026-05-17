/// Reverse of `RecordsParser::uncrypt`: encode an i64 value + order
/// into the obfuscated 8-byte sequence stored in HISCORE.SKI.
///
/// The encryption shifts ASCII digit characters by +21:
/// `'0'` (48) → `'E'` (69), `'1'` (49) → `'F'` (70), ..., `'9'` (57) → `'N'` (78).
///
/// The `high_extra` byte can be 0..=180 (values > 127 are non-UTF-8, so
/// the output is `Vec<u8>`, not `String`). Callers writing to text files
/// should treat this as raw byte output.
#[must_use] 
pub fn crypt(value: i64, order: usize) -> Vec<u8> {
    let low_5digits = (value % 100_000) as usize;
    let high_extra = if value >= 100_000 {
        let hundred_k_blocks = value / 100_000;
        75u8.wrapping_add(hundred_k_blocks as u8)
    } else {
        0
    };

    let digits = format!("{low_5digits:05}");
    let enc_digit = |i: usize| digits.as_bytes()[i] + 21;

    let value_check = 68 + 2 * (((value % 7) as u8) ^ 1);
    let order_check = 65 + (((order as u8) ^ 33) % 19);

    // Encrypted byte layout (positions 0-7):
    //   0 = least significant digit, 1 = value checksum, 2 = order checksum,
    //   3 = 4th digit, 4 = high_extra (100k blocks indicator),
    //   5 = 3rd digit, 6 = 2nd digit, 7 = most significant digit
    vec![
        enc_digit(4),
        value_check,
        order_check,
        enc_digit(3),
        high_extra,
        enc_digit(2),
        enc_digit(1),
        enc_digit(0),
    ]
}

/// Convenience: convert crypt output to str for use with `uncrypt`.
/// Only safe when `high_extra < 128`, i.e. `value < 5_300_000`.
/// Panics otherwise (single bytes >= 128 are not valid UTF-8).
#[must_use] 
pub fn crypt_str(value: i64, order: usize) -> String {
    let bytes = crypt(value, order);
    String::from_utf8(bytes).expect("crypt output should be valid UTF-8 for this value range")
}

fn str_hash(text: &str, seed: u32) -> u32 {
    let mut running_hash = 0u16;
    for (idx, ch) in text.chars().enumerate() {
        let ord_val = ch as u16;
        let multiplier = ((idx + 1) % 5) as u16 + 41;
        running_hash = running_hash.wrapping_add(ord_val.wrapping_mul(multiplier));
    }
    let seed16 = seed as u16;
    let result = running_hash
        .wrapping_mul(seed16.wrapping_rem(7).wrapping_add(1))
        .wrapping_add(seed16);
    u32::from(result)
}

/// Pascal `ProfileCode` — checksum for a PLAYERS.SKI profile entry.
/// Fields are passed positionally matching the Pascal `Profile_type` order.
#[allow(clippy::too_many_arguments)]
#[must_use] 
pub fn profile_code(
    name: &str,
    bestresult: &str,
    best4result: &str,
    suitcolor: u8,
    skicolor: u8,
    kothlevel: u8,
    replace: u8,
    bestwchill: u8,
    bestwcjump: u16,
    besthill: u8,
    bestjump: u16,
    bestpoints: u16,
    best4points: u16,
    cstyle: u8,
    wcs: i32,
    legswon: i32,
    wcswon: i32,
    totaljumps: i32,
) -> i32 {
    let concat = format!("{name}{bestresult}{best4result}");
    let mut code: i32 = str_hash(&concat, 11) as i32;

    code += i32::from(suitcolor ^ 31);
    code += i32::from(skicolor ^ 53);
    code += i32::from(kothlevel & 44);
    code += i32::from(replace | 91);
    code += i32::from(bestwchill ^ 157);
    code += i32::from(bestwcjump & 311);
    code += i32::from(besthill ^ 113);
    code += i32::from(bestjump & 277);
    code += i32::from(bestpoints | 133);
    code += i32::from(best4points & 31);

    code += i32::from(cstyle ^ 37);
    code += wcs & 741;
    code += legswon | 453;
    code += wcswon ^ 857;
    code += totaljumps + 5;

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypt_roundtrip() {
        // Safe range for ASCII round-trip: high_extra < 128
        // high_extra = 75 + (val / 100_000), so val < 53 * 100_000 = 5_300_000
        for val in [0i64, 1, 42, 99999, 100_000, 123_456, 5_299_999] {
            for order in 1..=5 {
                let enc_bytes = crypt(val, order);
                assert_eq!(enc_bytes.len(), 8, "crypt output must be 8 bytes for val={val} order={order}");
                let enc = String::from_utf8(enc_bytes).unwrap();
                let dec = super::super::super::parsers::records::uncrypt(&enc, order);
                assert_eq!(dec, val, "round-trip failed for val={val} order={order}: enc={enc:?}");
            }
        }
    }

    #[test]
    fn valuestr_matches_pascal() {
        let r = str_hash("SKI JUMPER0 (-)-", 11);
        assert_eq!(r, 13468);
    }

    #[test]
    fn profile_code_matches_pascal() {
        // From the example PLAYERS.SKI in the repo
        let code = profile_code(
            "SKI JUMPER", "0 (-)", "-",
            0, 0, 0, 0, 1, 930, 1, 930, 0, 0,
            1, 0, 0, 0, 1,
        );
        assert_eq!(code, 15942);
    }
}
