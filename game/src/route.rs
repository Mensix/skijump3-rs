#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Quit,
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    Jump,
    CompetitionJump,
    TeamCup,
    CustomCupSetup,
    Replays,
    ReplayPlayback,
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
    Back,
}
