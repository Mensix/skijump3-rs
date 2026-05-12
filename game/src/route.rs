#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    JumpMenu,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
}
