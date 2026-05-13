#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
}
