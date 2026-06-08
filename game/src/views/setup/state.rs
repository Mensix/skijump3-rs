use crate::text::lang::LangBase;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetupModal {
    WindPlace(usize),
    SeeComps(usize),
    ConfirmReset(u8),
    LanguagePicker(usize),
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
