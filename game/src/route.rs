#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    Jump,
    Replays,
    ReplayPlayback,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
}
