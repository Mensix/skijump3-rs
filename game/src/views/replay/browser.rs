use crate::components::detail_panel::paint_detail_panel;
use crate::components::layout::MainLayout;
use crate::components::modal::alert_prompt;
use crate::components::page_nav::cycle_index;
use crate::files::FileStore;
use crate::gfx::theme::{BG_DARK, FONT_BODY, FONT_GOLD};
use crate::jump::replay::ReplayTrace;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{Resources, ResourcesRef};
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};
use std::path::Path;

#[derive(Debug, Clone)]
struct ReplayEntry {
    filename: String,
    trace: Option<ReplayTrace>,
}

pub struct ReplayBrowserView {
    resources: ResourcesRef,
    entries: Vec<ReplayEntry>,
    selected: usize,
    confirm_delete: bool,
}

impl ReplayBrowserView {
    pub fn new(resources: ResourcesRef) -> Self {
        let entries = load_replays(&resources.files);
        Self {
            resources,
            entries,
            selected: 0,
            confirm_delete: false,
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

    fn delete_selected(&mut self) {
        let Some(filename) = self
            .selected_entry()
            .map(|entry| format!("{}.SJR", entry.filename))
        else {
            self.confirm_delete = false;
            return;
        };
        self.resources.files.delete_save(&filename);
        self.entries = load_replays(&self.resources.files);
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.confirm_delete = false;
    }
}

impl GameScreen for ReplayBrowserView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.confirm_delete {
            match event {
                UiEvent::Text('y' | 'Y') => self.delete_selected(),
                UiEvent::Text('n' | 'N') | UiEvent::KeyDown(Key::Escape) => {
                    self.confirm_delete = false;
                }
                _ => {}
            }
            nav.consume();
            return;
        }

        match event {
            UiEvent::KeyDown(Key::Escape) => {
                cx.state.selected_main_menu = 5;
                nav.back();
            }
            UiEvent::KeyDown(Key::Right | Key::Down) | UiEvent::Text(' ' | '+') => {
                self.move_next();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Left | Key::Up) | UiEvent::Text('-') => {
                self.move_prev();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Enter) => {
                if let Some(trace) = self.selected_entry().and_then(|entry| entry.trace.clone()) {
                    cx.state.selected_replay = Some(trace);
                    nav.navigate(RouteTarget::ReplayPlayback);
                }
            }
            UiEvent::KeyDown(Key::Delete) => {
                if !self.entries.is_empty() {
                    self.confirm_delete = true;
                }
                nav.consume();
            }
            UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => {}
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        paint.fill((11, 80, 100, 6), BG_DARK);
        paint.text((11, 80), FONT_GOLD, cx.layout.langbase.tr(17));
        paint_replay_menu(paint, cx.layout);
        cx.layout.footer(paint);
        paint_replay_panel(paint, &self.resources, &self.entries, self.selected);
        if self.confirm_delete {
            let are_you_sure = cx.layout.langbase.tr(193);
            paint_delete_confirm(
                paint,
                &self.entries[self.selected].filename,
                are_you_sure,
            );
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_delete_confirm(cx: &mut PaintCx<'_>, filename: &str, message: &str) {
    alert_prompt(
        cx,
        format!("Delete {filename}.SJR?"),
        message, true);
}

fn paint_replay_panel(
    cx: &mut PaintCx<'_>,
    resources: &Resources,
    entries: &[ReplayEntry],
    selected: usize,
) {
    let langbase = &resources.langbase;
    let entry = &entries[selected];
    let trace = entry.trace.as_ref().unwrap();

    let hill = resources.hills.hill(trace.meta.hill_idx).map_or_else(
        || "?".to_string(),
        |hill| format!("{} K{}", hill.name, hill.kr),
    );
    let field_pairs = vec![
        (langbase.tr(291).to_string(), trace.meta.author.clone()),
        (langbase.tr(292).to_string(), trace.meta.name.clone()),
        (langbase.tr(294).to_string(), hill),
    ];
    let extra_saved = Some(trace.meta.saved_at.clone());

    paint_detail_panel(
        cx,
        &format!("{}:", langbase.tr(25)),
        &entry.filename,
        &field_pairs,
        extra_saved.as_deref(),
        Some((selected + 1, entries.len())),
        langbase.tr(146),
        langbase.tr(290),
        entries.is_empty(),
    );
}

fn paint_replay_menu(cx: &mut PaintCx<'_>, layout: &MainLayout) {
    for (i, label) in [20, 21, 22, 23, 24, 25, 26].iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + if i == 6 { 12 } else { 0 };
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.tr(*label)),
        );
    }
}

fn load_replays(files: &FileStore) -> Vec<ReplayEntry> {
    let names = files.list_by_ext_all("SJR");
    names
        .into_iter()
        .map(|filename| {
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&filename)
                .to_string();
            let intro = stem.eq_ignore_ascii_case("INTRO");
            let data = files.read(&filename);
            let trace = ReplayTrace::from_sjr_bytes(&data, intro);
            ReplayEntry {
                filename: stem,
                trace: Some(trace),
            }
        })
        .collect()
}
