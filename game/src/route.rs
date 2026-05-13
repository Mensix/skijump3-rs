#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    Jump,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
}
