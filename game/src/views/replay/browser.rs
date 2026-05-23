use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_ERASE, FONT_DEFAULT, FONT_GOLD, FONT_HEADER, FONT_HELP};
use crate::jump::replay::ReplayTrace;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Component, Element, Event, Key, View};

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
    menu: Menu,
    entries: Vec<ReplayEntry>,
    selected: usize,
}

impl ReplayBrowserView {
    pub fn new(resources: ResourcesRef, store: StoreRef, layout: MainLayout) -> Self {
        let entries = load_replays(&resources.files);
        let items = vec![
            MenuItem {
                num: 1,
                label: 20,
                y_off: 0,
            },
            MenuItem {
                num: 2,
                label: 21,
                y_off: 0,
            },
            MenuItem {
                num: 3,
                label: 22,
                y_off: 0,
            },
            MenuItem {
                num: 4,
                label: 23,
                y_off: 0,
            },
            MenuItem {
                num: 5,
                label: 24,
                y_off: 0,
            },
            MenuItem {
                num: 6,
                label: 25,
                y_off: 0,
            },
            MenuItem {
                num: 0,
                label: 26,
                y_off: 12,
            },
        ];
        let menu = Menu::new(
            11,
            97,
            108,
            12,
            items,
            &layout.langbase,
            FONT_DEFAULT,
            FONT_DEFAULT,
        )
        .with_box(false);
        Self {
            resources,
            store,
            layout,
            menu,
            entries,
            selected: 0,
        }
    }

    fn selected_entry(&self) -> Option<&ReplayEntry> {
        self.entries.get(self.selected)
    }

    const fn move_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1) % self.entries.len();
        }
    }

    const fn move_prev(&mut self) {
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
        let mut els = vec![self.layout.background_element()];
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(17),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        els.extend(self.menu.elements());
        els.extend(self.layout.footer());

        // Pascal clearscreen: right panel background with dither + labels
        els.push(Element::fillbox(145, 50, 174, 149, 243));
        els.push(Element::fillbox(128, 70, 17, 129, 243));
        els.push(Element::FillArea { thing: 64 });
        els.push(Element::text(
            format!("{}:", self.resources.langbase.lstr(25)),
            170,
            51,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(146),
            150,
            185,
            FONT_HELP,
            false,
        ));

        if self.entries.is_empty() {
            els.push(Element::text(
                self.resources.langbase.lstr(290),
                170,
                80,
                FONT_GOLD,
                false,
            ));
            return els;
        }

        let entry = self.selected_entry().expect("selected replay");
        els.push(Element::text(
            format!("{}/{}", self.selected + 1, self.entries.len()),
            272,
            85,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(293),
            150,
            71,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(291),
            150,
            106,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(292),
            150,
            126,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(
            self.resources.langbase.lstr(294),
            150,
            146,
            FONT_HELP,
            false,
        ));
        els.push(Element::fillbox(163, 78, 95, 21, 248));
        els.push(Element::fillbox(164, 79, 93, 19, 243));
        els.push(Element::text(&entry.filename, 170, 85, FONT_GOLD, false));

        if let Some(trace) = &entry.trace {
            let hill = self.resources.hills.hill(trace.meta.hill_idx).map_or_else(
                || "?".to_string(),
                |hill| format!("{} K{}", hill.name, hill.kr),
            );
            els.push(Element::text(
                &trace.meta.author,
                170,
                115,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::text(
                &trace.meta.name,
                170,
                135,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::text(hill, 170, 155, FONT_DEFAULT, false));
            els.push(Element::text(
                &trace.meta.saved_at,
                170,
                163,
                FONT_HELP,
                false,
            ));
        } else if let Some(error) = &entry.error {
            els.push(Element::text("Unknown", 170, 115, FONT_HELP, false));
            els.push(Element::text(
                "Not a valid replay.",
                170,
                135,
                FONT_HELP,
                false,
            ));
            els.push(Element::text(error, 170, 155, FONT_HELP, false));
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => {
                self.store.selected_main_menu.set(5);
                Some(RouteTarget::Back)
            }
            Event::Keyboard(Key::Right | Key::Down | Key::Char(' ' | '+')) => {
                self.move_next();
                None
            }
            Event::Keyboard(Key::Left | Key::Up | Key::Char('-')) => {
                self.move_prev();
                None
            }
            Event::Keyboard(Key::Enter) => {
                let trace = self.selected_entry()?.trace.clone()?;
                self.store.replay_selection.select(trace);
                Some(RouteTarget::ReplayPlayback)
            }
            Event::Keyboard(_) => None,
        }
    }
}

fn load_replays(files: &crate::save::files::FileStore) -> Vec<ReplayEntry> {
    let names = match files.list_by_ext_all("SJR") {
        Ok(n) => n,
        Err(_) => return Vec::new(),
    };
    names
        .into_iter()
        .map(|filename| {
            let stem = filename
                .strip_suffix(".SJR")
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
