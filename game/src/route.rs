#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    Jump,
    CompetitionJump,
    CustomCupSetup,
    Replays,
    ReplayPlayback,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
    KothSetup,
    Back,
}
