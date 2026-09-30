use crate::jump::replay::ReplayTrace;
use crate::ui::ScreenEventCx;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    MainMenu,
    Welcome,
    JumpMenu,
    Practice,
    CompetitionJump,
    CustomCupSetup,
    Replays,
    ReplayPlayback {
        trace: Box<ReplayTrace>,
        return_to: ReplayReturn,
    },
    HallOfFame,
    HillRecords,
    OptionsMenu,
    ProfilesList,
    KothSetup,
    KothHillPicker,
    HillMakerSetup,
    EditHill(Option<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayReturn {
    MainMenu,
    Browser,
}

impl ReplayReturn {
    pub fn navigate(self, nav: &mut ScreenEventCx<RouteTarget>) {
        match self {
            Self::MainMenu => nav.navigate(RouteTarget::MainMenu),
            Self::Browser => nav.back(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::NavAction;

    #[test]
    fn replay_browser_return_pops_history() {
        let mut nav = ScreenEventCx::default();
        ReplayReturn::Browser.navigate(&mut nav);
        assert_eq!(nav.take_action(), NavAction::Back);
    }

    #[test]
    fn intro_return_navigates_to_main_menu() {
        let mut nav = ScreenEventCx::default();
        ReplayReturn::MainMenu.navigate(&mut nav);
        assert_eq!(
            nav.take_action(),
            NavAction::Navigate(RouteTarget::MainMenu)
        );
    }
}
