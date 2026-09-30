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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetupItem {
    Screen(usize),
    ConfigureKeys,
    HillGoals,
    HillMaker,
    Language,
    SoundEffects,
    GraphicsDetail,
    NameSet,
    TrainingRounds,
    ExtraStatistics,
    EventGap,
    WorldCupGap,
    CompactResults,
    InvisibleBackground,
    AutoRecordReplay,
    Goals,
    VisibleComputers,
    WindPosition,
    KoSystem,
    ComputerHillRecords,
    UniqueComputerNames,
    ResetBundledRecords,
    ClearRecords,
    ResetConfig,
}

impl SetupItem {
    pub(crate) const fn label_id(self) -> usize {
        match self {
            Self::Screen(1) => 196,
            Self::Screen(2) => 197,
            Self::Screen(3) => 198,
            Self::ConfigureKeys => 199,
            Self::HillGoals => 200,
            Self::HillMaker => 201,
            Self::Language => 204,
            Self::SoundEffects => 205,
            Self::GraphicsDetail => 206,
            Self::NameSet => 207,
            Self::TrainingRounds => 212,
            Self::ExtraStatistics => 213,
            Self::EventGap => 214,
            Self::WorldCupGap => 215,
            Self::CompactResults => 216,
            Self::InvisibleBackground => 217,
            Self::AutoRecordReplay => 218,
            Self::Goals => 219,
            Self::VisibleComputers => 220,
            Self::WindPosition => 221,
            Self::KoSystem => 222,
            Self::ComputerHillRecords => 226,
            Self::UniqueComputerNames => 227,
            Self::ResetBundledRecords => 228,
            Self::ClearRecords => 229,
            Self::ResetConfig => 230,
            Self::Screen(_) => unreachable!(),
        }
    }
}

const MAIN_ITEMS: &[SetupItem] = &[
    SetupItem::Screen(1),
    SetupItem::Screen(2),
    SetupItem::Screen(3),
    SetupItem::ConfigureKeys,
    SetupItem::HillGoals,
    SetupItem::HillMaker,
];
const DISPLAY_ITEMS: &[SetupItem] = &[
    SetupItem::Language,
    SetupItem::SoundEffects,
    SetupItem::GraphicsDetail,
    SetupItem::NameSet,
];
const GAME_ITEMS: &[SetupItem] = &[
    SetupItem::TrainingRounds,
    SetupItem::ExtraStatistics,
    SetupItem::EventGap,
    SetupItem::WorldCupGap,
    SetupItem::CompactResults,
    SetupItem::InvisibleBackground,
    SetupItem::AutoRecordReplay,
    SetupItem::Goals,
    SetupItem::VisibleComputers,
    SetupItem::WindPosition,
    SetupItem::KoSystem,
];
const RESET_ITEMS: &[SetupItem] = &[
    SetupItem::ComputerHillRecords,
    SetupItem::UniqueComputerNames,
    SetupItem::ResetBundledRecords,
    SetupItem::ClearRecords,
    SetupItem::ResetConfig,
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct SetupPage {
    pub title_id: usize,
    pub trailing_label_id: usize,
    pub items: &'static [SetupItem],
}

pub(crate) fn setup_page(screen: usize) -> Option<SetupPage> {
    let (title_id, trailing_label_id, items) = match screen {
        0 => (175, 195, MAIN_ITEMS),
        1 => (176, 203, DISPLAY_ITEMS),
        2 => (177, 211, GAME_ITEMS),
        3 => (178, 225, RESET_ITEMS),
        _ => return None,
    };
    Some(SetupPage {
        title_id,
        trailing_label_id,
        items,
    })
}

pub(crate) fn hex_char(index: usize) -> &'static str {
    "0123456789ABCDEF".get(index..index + 1).unwrap_or("?")
}

pub(crate) fn wind_place_name(lang: &LangBase, place: usize) -> String {
    match place {
        0 => format!("{}-{}", lang.tr(392), lang.tr(393)),
        1 => format!("{}-{}", lang.tr(391), lang.tr(393)),
        2 => format!("{}-{}", lang.tr(392), lang.tr(395)),
        3 => format!("{}-{}", lang.tr(392), lang.tr(394)),
        4 => format!("{}-{}", lang.tr(391), lang.tr(395)),
        5 => format!("{}-{}", lang.tr(390), lang.tr(395)),
        6 => format!("{}-{}", lang.tr(390), lang.tr(394)),
        7 => format!("{}-{}", lang.tr(390), lang.tr(393)),
        8 => format!("{}: {}", lang.tr(396), lang.tr(390)),
        9 => format!("{}: {}", lang.tr(396), lang.tr(391)),
        10 => format!("{}: {}", lang.tr(396), lang.tr(392)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jump::wind::WIND_POSITION_COUNT;

    #[test]
    fn all_wind_place_labels_match_pascal_menu_order() {
        let mut strings = vec!["?".to_string(); 397];
        for (index, value) in [
            (390, "Top"),
            (391, "Middle"),
            (392, "Bottom"),
            (393, "Left"),
            (394, "Center"),
            (395, "Right"),
            (396, "Jumper"),
        ] {
            strings[index] = value.to_string();
        }
        let lang = LangBase::new(vec![strings], Vec::new(), 0);
        let actual: Vec<String> = (0..usize::from(WIND_POSITION_COUNT))
            .map(|place| wind_place_name(&lang, place))
            .collect();

        assert_eq!(
            actual,
            [
                "Bottom-Left",
                "Middle-Left",
                "Bottom-Right",
                "Bottom-Center",
                "Middle-Right",
                "Top-Right",
                "Top-Center",
                "Top-Left",
                "Jumper: Top",
                "Jumper: Middle",
                "Jumper: Bottom",
            ]
        );
    }

    #[test]
    fn setup_metadata_defines_counts_and_labels() {
        let pages: Vec<_> = (0..4).map(|screen| setup_page(screen).unwrap()).collect();
        assert_eq!(
            pages
                .iter()
                .map(|page| page.items.len())
                .collect::<Vec<_>>(),
            [6, 4, 11, 5]
        );
        assert_eq!(
            pages
                .iter()
                .map(|page| page.trailing_label_id)
                .collect::<Vec<_>>(),
            [195, 203, 211, 225]
        );
        assert_eq!(pages[2].items[10], SetupItem::KoSystem);
        assert_eq!(pages[2].items[10].label_id(), 222);
    }
}
