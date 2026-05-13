#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
}
