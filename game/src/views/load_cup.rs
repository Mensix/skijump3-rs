use crate::components::layout::MainLayout;
use crate::components::page_nav::cycle_index;
use crate::gfx::theme::{BG_DARK, BG_PURPLE, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY};
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
        match self.save_manager.load_cup(&filename) {
            Ok(data) => {
                cx.state.active_competition = Some(data.active);
                nav.navigate(RouteTarget::CompetitionJump);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
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
        paint.text((11, 80), FONT_GOLD, cx.layout.langbase.lstr(17));
        paint_main_menu(paint, &cx.layout);
        cx.layout.footer(paint);
        paint_panel(
            paint,
            &self.entries,
            self.selected,
            self.error.as_deref(),
            cx.layout.langbase.lstr(520),
            cx.layout.langbase.lstr(523),
        );
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_main_menu(cx: &mut PaintCx<'_>, layout: &MainLayout) {
    for (i, label) in [20, 21, 22, 23, 24, 25, 520, 26].iter().enumerate() {
        let num = if i == 7 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + if i == 7 { 12 } else { 0 };
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
}

fn paint_panel(
    cx: &mut PaintCx<'_>,
    entries: &[CupSaveEntry],
    selected: usize,
    error: Option<&str>,
    title: &str,
    empty_text: &str,
) {
    cx.pattern_fill((145, 50, 174, 149), BG_PURPLE);
    cx.pattern_fill((128, 70, 17, 129), BG_PURPLE);
    cx.text((170, 51), FONT_GRAY, format!("{}:", title));
    if entries.is_empty() {
        cx.text((170, 80), FONT_GOLD, empty_text);
        return;
    }
    let entry = &entries[selected];
    cx.text(
        (272, 85),
        FONT_GRAY,
        format!("{}/{}", selected + 1, entries.len()),
    );
    cx.fill((163, 78, 120, 21), FILL_PURPLE);
    cx.fill((164, 79, 118, 19), BG_PURPLE);
    cx.text((170, 85), FONT_GOLD, &entry.filename);
    cx.text((170, 115), FONT_BODY, &entry.title);
    cx.text((170, 135), FONT_GRAY, format!("{:?}", entry.kind));
    cx.text((170, 155), FONT_GRAY, &entry.saved_at);
    if let Some(error) = error {
        cx.text((150, 180), FONT_GRAY, error);
    }
}
