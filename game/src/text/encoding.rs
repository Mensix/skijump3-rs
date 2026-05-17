const fn cp850_to_char(b: u8) -> char {
    match b {
        0x80 => '\u{00C7}',
        0x81 => '\u{00FC}',
        0x82 => '\u{00E9}',
        0x83 => '\u{00E2}',
        0x84 => '\u{00E4}',
        0x85 => '\u{00E0}',
        0x86 => '\u{00E5}',
        0x87 => '\u{00E7}',
        0x88 => '\u{00EA}',
        0x89 => '\u{00EB}',
        0x8A => '\u{00E8}',
        0x8B => '\u{00EF}',
        0x8C => '\u{00EE}',
        0x8D => '\u{00EC}',
        0x8E => '\u{00C4}',
        0x8F => '\u{00C5}',
        0x90 => '\u{00C9}',
        0x91 => '\u{00E6}',
        0x92 => '\u{00C6}',
        0x93 => '\u{00F4}',
        0x94 => '\u{00F6}',
        0x95 => '\u{00F2}',
        0x96 => '\u{00FB}',
        0x97 => '\u{00F9}',
        0x98 | 0xFF => '\u{00FF}',
        0x99 => '\u{00D6}',
        0x9A => '\u{00DC}',
        0x9B => '\u{00F8}',
        0x9C => '\u{00A3}',
        0x9D => '\u{00D8}',
        0x9E => '\u{00D7}',
        0x9F => '\u{0192}',
        0xA0 => '\u{00E1}',
        0xA1 => '\u{00ED}',
        0xA2 => '\u{00F3}',
        0xA3 => '\u{00FA}',
        0xA4 => '\u{00F1}',
        0xA5 => '\u{00D1}',
        0xA6 => '\u{00AA}',
        0xA7 => '\u{00BA}',
        0xA8 => '\u{00BF}',
        0xE1 => '\u{00DF}',
        0xE6 => '\u{00B5}',
        _ => {
            if b.is_ascii() {
                b as char
            } else {
                '\u{FFFD}'
            }
        }
    }
}

#[must_use] 
pub fn decode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        out.push(cp850_to_char(b));
    }
    out
}

const fn char_to_cp850(c: char) -> u8 {
    match c {
        '\u{00C7}' => 0x80,
        '\u{00FC}' => 0x81,
        '\u{00E9}' => 0x82,
        '\u{00E2}' => 0x83,
        '\u{00E4}' => 0x84,
        '\u{00E0}' => 0x85,
        '\u{00E5}' => 0x86,
        '\u{00E7}' => 0x87,
        '\u{00EA}' => 0x88,
        '\u{00EB}' => 0x89,
        '\u{00E8}' => 0x8A,
        '\u{00EF}' => 0x8B,
        '\u{00EE}' => 0x8C,
        '\u{00EC}' => 0x8D,
        '\u{00C4}' => 0x8E,
        '\u{00C5}' => 0x8F,
        '\u{00C9}' => 0x90,
        '\u{00E6}' => 0x91,
        '\u{00C6}' => 0x92,
        '\u{00F4}' => 0x93,
        '\u{00F6}' => 0x94,
        '\u{00F2}' => 0x95,
        '\u{00FB}' => 0x96,
        '\u{00F9}' => 0x97,
        '\u{00FF}' => 0x98,
        '\u{00D6}' => 0x99,
        '\u{00DC}' => 0x9A,
        '\u{00F8}' => 0x9B,
        '\u{00A3}' => 0x9C,
        '\u{00D8}' => 0x9D,
        '\u{00D7}' => 0x9E,
        '\u{0192}' => 0x9F,
        '\u{00E1}' => 0xA0,
        '\u{00ED}' => 0xA1,
        '\u{00F3}' => 0xA2,
        '\u{00FA}' => 0xA3,
        '\u{00F1}' => 0xA4,
        '\u{00D1}' => 0xA5,
        '\u{00AA}' => 0xA6,
        '\u{00BA}' => 0xA7,
        '\u{00BF}' => 0xA8,
        '\u{00DF}' => 0xE1,
        '\u{00B5}' => 0xE6,
        _ => {
            if c.is_ascii() {
                c as u8
            } else {
                b'?'
            }
        }
    }
}

pub fn encode(text: &str) -> Vec<u8> {
    text.chars().map(char_to_cp850).collect()
}
