#[derive(Clone, Debug)]
pub enum Cmd {
    None,
    Navigate(RouteTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteTarget {
    MainMenu,
    RaceMenu,
    OptionsMenu,
    Play(u8),
    Results { score: u32, hill: u8 },
}

impl RouteTarget {
    pub fn as_cmd(self) -> Cmd {
        Cmd::Navigate(self)
    }
}