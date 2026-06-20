use crate::components::detail_panel::paint_detail_panel;
use crate::components::layout::MainLayout;
use crate::components::page_nav::cycle_index;
use crate::gfx::theme::{BG_DARK, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::save::cup::CupSaveEntry;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub struct LoadCupView {
    save_manager: SaveRef,
    entries: Vec<CupSaveEntry>,
    selected: usize,
    error: Option<String>,
}

impl LoadCupView {
    pub fn new(save_manager: SaveRef) -> Self {
        let entries = save_manager.list_cup_saves();
        Self {
            save_manager,
            entries,
            selected: 0,
            error: None,
        }
    }

    fn selected_entry(&self) -> Option<&CupSaveEntry> {
        self.entries.get(self.selected)
    }

    fn move_next(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), 1);
    }

    fn move_prev(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), -1);
    }

    fn load_selected(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>) {
        let Some(filename) = self.selected_entry().map(|entry| entry.filename.clone()) else {
            return;
        };
        let data = self.save_manager.load_cup(&filename);
        cx.state.active_competition = Some(data.active);
        nav.navigate(RouteTarget::CompetitionJump);
    }
}

impl GameScreen for LoadCupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::KeyDown(Key::Escape) => nav.back(),
            UiEvent::KeyDown(Key::Right | Key::Down) | UiEvent::Text(' ' | '+') => {
                self.move_next();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Left | Key::Up) | UiEvent::Text('-') => {
                self.move_prev();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Enter) => self.load_selected(cx, nav),
            UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => {}
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((11, 80, 100, 6), BG_DARK);
        paint.text((11, 80), FONT_GOLD, cx.layout.langbase.lstr(18));
        paint_jump_menu(paint, cx.layout);
        cx.layout.footer(paint);
        paint_detail_panel(
            paint,
            &format!("{}:", cx.layout.langbase.lstr(520)),
            self.selected_entry().map_or("", |e| &e.filename),
            &[
                (
                    "Cup:".to_string(),
                    self.selected_entry().map_or("", |e| &e.title).to_string(),
                ),
                (
                    "Saved:".to_string(),
                    self.selected_entry()
                        .map_or("", |e| &e.saved_at)
                        .to_string(),
                ),
            ],
            None,
            None,
            cx.layout.langbase.lstr(146),
            cx.layout.langbase.lstr(523),
            self.entries.is_empty(),
            self.error.as_deref(),
        );
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_jump_menu(cx: &mut PaintCx<'_>, layout: &MainLayout) {
    for (i, label) in [27, 28, 29, 30, 31, 32, 520, 33].iter().enumerate() {
        let num = if i == 7 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + if i == 7 { 12 } else { 0 };
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
}
