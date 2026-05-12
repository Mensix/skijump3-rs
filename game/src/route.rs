#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    JumpMenu,
    OptionsMenu,
    Profiles,
}
