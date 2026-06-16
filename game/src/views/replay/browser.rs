use crate::components::layout::MainLayout;
use crate::components::page_nav::cycle_index;
use crate::files::FileStore;
use crate::gfx::theme::{BG_DARK, BG_PURPLE, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::jump::replay::ReplayTrace;
use crate::route::RouteTarget;
use crate::store::{GameStateRef, Resources, ResourcesRef};
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};
use std::path::Path;

#[derive(Debug, Clone)]
struct ReplayEntry {
    filename: String,
    trace: Option<ReplayTrace>,
    error: Option<String>,
}

pub struct ReplayBrowserView {
    resources: ResourcesRef,
    store: GameStateRef,
    layout: MainLayout,
    entries: Vec<ReplayEntry>,
    selected: usize,
}

impl ReplayBrowserView {
    pub fn new(resources: ResourcesRef, store: GameStateRef, layout: MainLayout) -> Self {
        let entries = load_replays(&resources.files);
        Self {
            resources,
            store,
            layout,
            entries,
            selected: 0,
        }
    }

    fn selected_entry(&self) -> Option<&ReplayEntry> {
        self.entries.get(self.selected)
    }

    fn move_next(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), 1);
    }

    fn move_prev(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), -1);
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        cx.fill((11, 80, 100, 6), BG_DARK);
        cx.text((11, 80), FONT_GOLD, self.layout.langbase.lstr(17));
        paint_replay_menu(cx, &self.layout);
        self.layout.footer(cx);
        paint_replay_panel(cx, &self.resources, &self.entries, self.selected);
    }
}

impl Screen<RouteTarget> for ReplayBrowserView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::KeyDown(Key::Escape) => {
                self.store.borrow_mut().selected_main_menu = 5;
                cx.back();
            }
            UiEvent::KeyDown(Key::Right | Key::Down) | UiEvent::Text(' ' | '+') => {
                self.move_next();
                cx.consume();
            }
            UiEvent::KeyDown(Key::Left | Key::Up) | UiEvent::Text('-') => {
                self.move_prev();
                cx.consume();
            }
            UiEvent::KeyDown(Key::Enter) => {
                if let Some(trace) = self.selected_entry().and_then(|entry| entry.trace.clone()) {
                    self.store.borrow_mut().selected_replay = Some(trace);
                    cx.navigate(RouteTarget::ReplayPlayback);
                }
            }
            UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_replay_panel(
    cx: &mut PaintCx<'_>,
    resources: &Resources,
    entries: &[ReplayEntry],
    selected: usize,
) {
    let langbase = &resources.langbase;

    // Pascal clearscreen: right panel background with dither + labels
    cx.pattern_fill((145, 50, 174, 149), BG_PURPLE);
    cx.pattern_fill((128, 70, 17, 129), BG_PURPLE);
    cx.text((170, 51), FONT_GRAY, format!("{}:", langbase.lstr(25)));
    cx.text((150, 185), FONT_GRAY, langbase.lstr(146));

    if entries.is_empty() {
        cx.text((170, 80), FONT_GOLD, langbase.lstr(290));
        return;
    }

    let entry = &entries[selected];
    cx.text(
        (272, 85),
        FONT_GRAY,
        format!("{}/{}", selected + 1, entries.len()),
    );
    cx.text((150, 71), FONT_GRAY, langbase.lstr(293));
    cx.text((150, 106), FONT_GRAY, langbase.lstr(291));
    cx.text((150, 126), FONT_GRAY, langbase.lstr(292));
    cx.text((150, 146), FONT_GRAY, langbase.lstr(294));
    cx.fill((163, 78, 95, 21), FILL_PURPLE);
    cx.fill((164, 79, 93, 19), BG_PURPLE);
    cx.text((170, 85), FONT_GOLD, &entry.filename);

    if let Some(trace) = &entry.trace {
        let hill = resources.hills.hill(trace.meta.hill_idx).map_or_else(
            || "?".to_string(),
            |hill| format!("{} K{}", hill.name, hill.kr),
        );
        cx.text((170, 115), FONT_BODY, &trace.meta.author);
        cx.text((170, 135), FONT_BODY, &trace.meta.name);
        cx.text((170, 155), FONT_BODY, hill);
        cx.text((170, 163), FONT_GRAY, &trace.meta.saved_at);
    } else if let Some(error) = &entry.error {
        cx.text((170, 115), FONT_GRAY, "Unknown");
        cx.text((170, 135), FONT_GRAY, "Not a valid replay.");
        cx.text((170, 155), FONT_GRAY, error);
    }
}

fn paint_replay_menu(cx: &mut PaintCx<'_>, layout: &MainLayout) {
    for (i, label) in [20, 21, 22, 23, 24, 25, 26].iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + if i == 6 { 12 } else { 0 };
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
}

fn load_replays(files: &FileStore) -> Vec<ReplayEntry> {
    let Ok(names) = files.list_by_ext_all("SJR") else {
        return Vec::new();
    };
    names
        .into_iter()
        .map(|filename| {
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&filename)
                .to_string();
            let intro = stem.eq_ignore_ascii_case("INTRO");
            match files
                .read(&filename)
                .map_err(|err| err.to_string())
                .and_then(|bytes| {
                    ReplayTrace::from_sjr_bytes(&bytes, intro).map_err(|err| format!("{err:?}"))
                }) {
                Ok(trace) => ReplayEntry {
                    filename: stem,
                    trace: Some(trace),
                    error: None,
                },
                Err(error) => ReplayEntry {
                    filename: stem,
                    trace: None,
                    error: Some(error),
                },
            }
        })
        .collect()
}
