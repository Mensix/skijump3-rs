#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    OptionsMenu,
    Profiles,
    Play(u8),
}
