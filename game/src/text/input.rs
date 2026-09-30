use crate::text::lang::LangBase;

pub(crate) fn key_name(code: i32, lang: &LangBase) -> String {
    if code == 0 {
        return "NULL".to_string();
    }
    let hi = ((code >> 8) & 0xff) as u8;
    let lo = (code & 0xff) as u8;
    match (hi, lo) {
        (0, 59..=67) => format!("F{}", lo - 58),
        (0, 71) => "HOME".to_string(),
        (0, 72) => lang.tr(280).to_string(),
        (0, 73) => "PAGE UP".to_string(),
        (0, 75) => lang.tr(281).to_string(),
        (0, 76) => "NP 5".to_string(),
        (0, 77) => lang.tr(282).to_string(),
        (0, 79) => "END".to_string(),
        (0, 80) => lang.tr(283).to_string(),
        (0, 81) => "PAGE DOWN".to_string(),
        (0, 82) => "INSERT".to_string(),
        (0, 83) => "DELETE".to_string(),
        (8, _) => "BACKSPACE".to_string(),
        (9, _) => "TAB".to_string(),
        (b' ', _) => "SPACE".to_string(),
        (b'.', _) => ".".to_string(),
        (b',', _) => ",".to_string(),
        (b'-', _) => "-".to_string(),
        (b'+', _) => "+".to_string(),
        (b'/', _) => "/".to_string(),
        (b'*', _) => "*".to_string(),
        (b'0'..=b'9', _) => char::from(hi).to_string(),
        (b'A'..=b'Z', _) => char::from(hi).to_string(),
        (b'a'..=b'z', _) => char::from(hi).to_ascii_uppercase().to_string(),
        _ => "NULL".to_string(),
    }
}
