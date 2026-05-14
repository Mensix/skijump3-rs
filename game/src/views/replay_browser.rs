use crate::components::layout::MainLayout;
use crate::jump::replay::ReplayTrace;
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Element, Event, Key, View};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct ReplayEntry {
    filename: String,
    trace: Option<ReplayTrace>,
    error: Option<String>,
}

pub struct ReplayBrowserView {
    resources: ResourcesRef,
    store: StoreRef,
    layout: MainLayout,
    entries: Vec<ReplayEntry>,
    selected: usize,
}

impl ReplayBrowserView {
    pub fn new(resources: ResourcesRef, store: StoreRef, layout: MainLayout) -> Self {
        let entries = load_replays();
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
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1) % self.entries.len();
        }
    }

    fn move_prev(&mut self) {
        if !self.entries.is_empty() {
            self.selected = if self.selected == 0 {
                self.entries.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

impl View<RouteTarget> for ReplayBrowserView {
    fn elements(&self) -> Vec<Element> {
        let mut els = self.layout.background();
        els.extend(self.layout.jumpers());
        els.extend(self.layout.registration());

        els.push(Element::fillbox(145, 50, 319, 199, BG_ERASE));
        els.push(Element::fillbox(128, 70, 145, 199, BG_ERASE));
        els.push(Element::fillbox(128, 70, 17, 129, 64));
        els.push(Element::fillbox(145, 50, 174, 149, 64));
        els.push(Element::text_color(
            format!("{}:", self.resources.langbase.lstr(25)),
            170,
            51,
            FONT_GOLD,
        ));
        els.push(Element::text_color(
            self.resources.langbase.lstr(146),
            150,
            185,
            FONT_GOLD,
        ));

        if self.entries.is_empty() {
            els.push(Element::text_color(
                self.resources.langbase.lstr(290),
                170,
                80,
                FONT_HEADER,
            ));
            return els;
        }

        let entry = self.selected_entry().expect("selected replay");
        els.push(Element::text_color(
            format!("{}/{}", self.selected + 1, self.entries.len()),
            272,
            85,
            FONT_DEFAULT,
        ));
        els.push(Element::text_color(
            self.resources.langbase.lstr(293),
            150,
            71,
            FONT_DEFAULT,
        ));
        els.push(Element::fillbox(163, 78, 95, 21, 248));
        els.push(Element::fillbox(164, 79, 93, 19, BG_ERASE));
        els.push(Element::text_color(&entry.filename, 170, 85, FONT_DEFAULT));

        els.push(Element::text_color(
            self.resources.langbase.lstr(291),
            150,
            106,
            FONT_DEFAULT,
        ));
        els.push(Element::text_color(
            self.resources.langbase.lstr(292),
            150,
            126,
            FONT_DEFAULT,
        ));
        els.push(Element::text_color(
            self.resources.langbase.lstr(294),
            150,
            146,
            FONT_DEFAULT,
        ));

        if let Some(trace) = &entry.trace {
            let hill = self
                .resources
                .hills
                .hill(trace.meta.hill_idx)
                .map(|hill| format!("{} K{}", hill.name, hill.kr))
                .unwrap_or_else(|| "?".to_string());
            let color = if trace.meta.valid_checksum {
                FONT_GREET
            } else {
                FONT_GOLD
            };
            els.push(Element::text_color(
                &trace.meta.author,
                170,
                115,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                &trace.meta.name,
                170,
                135,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(hill, 170, 155, color));
            els.push(Element::text_color(
                &trace.meta.saved_at,
                170,
                163,
                FONT_DEFAULT,
            ));
            if trace.meta.intro {
                els.push(Element::text_color("INTRO", 260, 115, FONT_GOLD));
            }
        } else if let Some(error) = &entry.error {
            els.push(Element::text_color("Unknown", 170, 115, FONT_DEFAULT));
            els.push(Element::text_color(
                "Not a valid replay.",
                170,
                135,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(error, 170, 155, FONT_GOLD));
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
            Event::Keyboard(Key::Right | Key::Down | Key::Char(' ') | Key::Char('+')) => {
                self.move_next();
                None
            }
            Event::Keyboard(Key::Left | Key::Up | Key::Char('-')) => {
                self.move_prev();
                None
            }
            Event::Keyboard(Key::Enter) => {
                let trace = self.selected_entry()?.trace.clone()?;
                *self.store.selected_replay.borrow_mut() = Some(trace);
                Some(RouteTarget::ReplayPlayback)
            }
            _ => None,
        }
    }
}

fn load_replays() -> Vec<ReplayEntry> {
    let mut paths = replay_paths();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let filename = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("?")
                .to_string();
            let intro = filename.eq_ignore_ascii_case("INTRO");
            match std::fs::read(&path)
                .map_err(|err| err.to_string())
                .and_then(|bytes| {
                    ReplayTrace::from_sjr_bytes(&bytes, intro).map_err(|err| format!("{err:?}"))
                }) {
                Ok(trace) => ReplayEntry {
                    filename,
                    trace: Some(trace),
                    error: None,
                },
                Err(error) => ReplayEntry {
                    filename,
                    trace: None,
                    error: Some(error),
                },
            }
        })
        .collect()
}

fn replay_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for dir in [Path::new("."), Path::new("game/assets"), Path::new("..")] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("SJR"))
                {
                    paths.push(path);
                }
            }
        }
    }
    paths
}
