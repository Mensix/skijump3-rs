use crate::components::layout::MainLayout;
use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveRef;
use crate::store::GameState;
use crate::ui::{ScreenBackground, ScreenEventCx, UiCanvas, UiEvent};

pub struct GameCx<'a> {
    pub state: &'a mut GameState,
    pub(crate) save_manager: SaveRef,
    pub layout: &'a MainLayout,
}

#[derive(Clone)]
pub struct Persistence {
    save_manager: SaveRef,
}

impl GameCx<'_> {
    pub fn persistence(&self) -> Persistence {
        Persistence {
            save_manager: std::rc::Rc::clone(&self.save_manager),
        }
    }
}

impl Persistence {
    pub fn from(save_manager: SaveRef) -> Self {
        Self { save_manager }
    }
    pub fn save_config(&self, config: &Config) -> bool {
        self.save_manager.save_config(config)
    }
    pub fn save_players(&self, profiles: &ProfileStore) -> bool {
        self.save_manager.save_players(profiles)
    }
    pub fn save_records(&self, records: &RecordStore) -> bool {
        self.save_manager.save_records(records)
    }
    pub fn write_file(&self, filename: &str, data: &[u8]) -> bool {
        self.save_manager.files.write(filename, data)
    }
}

pub trait GameScreen {
    fn update(&mut self, _: &mut GameCx<'_>) {}
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent);
    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas);
    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
    fn has_modal(&self) -> bool {
        false
    }
}

pub(crate) enum NavSignal {
    Route(RouteTarget),
    Back,
    None,
}

impl NavSignal {
    pub(crate) fn dispatch(self, nav: &mut ScreenEventCx<RouteTarget>) {
        match self {
            Self::Route(route) => nav.navigate(route),
            Self::Back => nav.back(),
            Self::None => nav.consume(),
        }
    }
}
