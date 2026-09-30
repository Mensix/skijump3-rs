const CP850_HIGH: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ',
    'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '×', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ',
    'ª', 'º', '¿', '®', '¬', '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À', '©',
    '╣', '║', '╗', '╝', '¢', '¥', '┐', '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã', '╚', '╔', '╩', '╦',
    '╠', '═', '╬', '¤', 'ð', 'Ð', 'Ê', 'Ë', 'È', 'ı', 'Í', 'Î', 'Ï', '┘', '┌', '█', '▄', '¦', 'Ì',
    '▀', 'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ', 'Þ', 'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´', '≡', '±',
    '‗', '¾', '¶', '§', '÷', '¸', '°', '¨', '·', '¹', '³', '²', '■', '\u{00A0}',
];

const fn cp850_to_char(b: u8) -> char {
    if b.is_ascii() {
        b as char
    } else {
        CP850_HIGH[(b - 0x80) as usize]
    }
}

pub fn decode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        out.push(cp850_to_char(b));
    }
    out
}

pub fn encode(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| {
            if c.is_ascii() {
                c as u8
            } else {
                CP850_HIGH
                    .iter()
                    .position(|&candidate| candidate == c)
                    .map_or(b'?', |idx| idx as u8 + 0x80)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp850_encoder_is_inverse_of_decoder() {
        let bytes: Vec<u8> = (0..=u8::MAX).collect();
        assert_eq!(encode(&decode(&bytes)), bytes);
    }

    #[test]
    fn unsupported_characters_use_dos_replacement() {
        assert_eq!(encode("A€B"), b"A?B");
    }
}
