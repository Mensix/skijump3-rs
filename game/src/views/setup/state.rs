use crate::text::lang::LangBase;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetupModal {
    WindPlace(usize),
    SeeComps(usize),
    ConfirmReset(u8),
    LanguagePicker(usize),
    ConfigureKeys {
        selected: usize,
        capture: Option<usize>,
    },
    NameSetInput,
    HillGoals(usize),
}

pub(crate) fn hex_char(index: usize) -> &'static str {
    match index {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "A",
        11 => "B",
        12 => "C",
        13 => "D",
        14 => "E",
        15 => "F",
        _ => "?",
    }
}

pub(crate) fn wind_place_name(langbase: &LangBase, place: usize) -> String {
    match place {
        1 => format!("{}-{}", langbase.lstr(392), langbase.lstr(393)),
        2 => format!("{}-{}", langbase.lstr(391), langbase.lstr(393)),
        3 => format!("{}-{}", langbase.lstr(392), langbase.lstr(395)),
        4 => format!("{}-{}", langbase.lstr(392), langbase.lstr(394)),
        5 => format!("{}-{}", langbase.lstr(391), langbase.lstr(395)),
        6 => format!("{}-{}", langbase.lstr(390), langbase.lstr(395)),
        7 => format!("{}-{}", langbase.lstr(390), langbase.lstr(394)),
        8 => format!("{}-{}", langbase.lstr(390), langbase.lstr(393)),
        11 => format!("{}: {}", langbase.lstr(396), langbase.lstr(390)),
        12 => format!("{}: {}", langbase.lstr(396), langbase.lstr(391)),
        13 => format!("{}: {}", langbase.lstr(396), langbase.lstr(392)),
        _ => unreachable!(),
    }
}

pub(crate) fn key_name(code: i32, langbase: &LangBase) -> String {
    if code == 0 {
        return "NULL".to_string();
    }
    let hi = ((code >> 8) & 0xff) as u8;
    let lo = (code & 0xff) as u8;
    match (hi, lo) {
        (0, 59..=67) => format!("F{}", lo - 58),
        (0, 71) => "HOME".to_string(),
        (0, 72) => langbase.lstr(280).to_string(),
        (0, 73) => "PAGE UP".to_string(),
        (0, 75) => langbase.lstr(281).to_string(),
        (0, 76) => "NP 5".to_string(),
        (0, 77) => langbase.lstr(282).to_string(),
        (0, 79) => "END".to_string(),
        (0, 80) => langbase.lstr(283).to_string(),
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
