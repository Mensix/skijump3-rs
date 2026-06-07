use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::components::page_nav::cycle_index;
use crate::files::FileStore;
use crate::gfx::palette::{
    BG_ERASE, BG_LEFT, FILL_BORDER, FONT_DEFAULT, FONT_GOLD, FONT_HEADER, FONT_HELP,
};
use crate::jump::replay::ReplayTrace;
use crate::route::RouteTarget;
use crate::store::{Resources, ResourcesRef, StoreRef};
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
            MenuItem::new(1, 20),
            MenuItem::new(2, 21),
            MenuItem::new(3, 22),
            MenuItem::new(4, 23),
            MenuItem::new(5, 24),
            MenuItem::new(6, 25),
            MenuItem::with_y(0, 26, 12),
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

    fn move_next(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), 1);
    }

    fn move_prev(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), -1);
    }
}

impl View<RouteTarget> for ReplayBrowserView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(17),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        els.extend(self.menu.elements());
        els.extend(self.layout.footer());
        els.extend(replay_panel_elements(
            &self.resources,
            &self.entries,
            self.selected,
        ));
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => {
                self.store.set_selected_main_menu(5);
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
                self.store.select_replay(trace);
                Some(RouteTarget::ReplayPlayback)
            }
            Event::Keyboard(_) => None,
        }
    }

    fn gpu_background(&self) -> engine::ui::BackgroundMode {
        engine::ui::BackgroundMode::MainPng
    }
}

fn replay_panel_elements(
    resources: &Resources,
    entries: &[ReplayEntry],
    selected: usize,
) -> Vec<Element> {
    let langbase = &resources.langbase;

    let mut els = Vec::new();

    // Pascal clearscreen: right panel background with dither + labels
    els.push(Element::fillbox(145, 50, 174, 149, BG_LEFT));
    els.push(Element::fillbox(128, 70, 17, 129, BG_LEFT));
    els.push(Element::fill_area(64));
    els.push(Element::text(
        format!("{}:", langbase.lstr(25)),
        170,
        51,
        FONT_HELP,
        false,
    ));
    els.push(Element::text(
        langbase.lstr(146),
        150,
        185,
        FONT_HELP,
        false,
    ));

    if entries.is_empty() {
        els.push(Element::text(langbase.lstr(290), 170, 80, FONT_GOLD, false));
        return els;
    }

    let entry = &entries[selected];
    els.push(Element::text(
        format!("{}/{}", selected + 1, entries.len()),
        272,
        85,
        FONT_HELP,
        false,
    ));
    els.push(Element::text(langbase.lstr(293), 150, 71, FONT_HELP, false));
    els.push(Element::text(
        langbase.lstr(291),
        150,
        106,
        FONT_HELP,
        false,
    ));
    els.push(Element::text(
        langbase.lstr(292),
        150,
        126,
        FONT_HELP,
        false,
    ));
    els.push(Element::text(
        langbase.lstr(294),
        150,
        146,
        FONT_HELP,
        false,
    ));
    els.push(Element::fillbox(163, 78, 95, 21, FILL_BORDER));
    els.push(Element::fillbox(164, 79, 93, 19, BG_LEFT));
    els.push(Element::text(&entry.filename, 170, 85, FONT_GOLD, false));

    if let Some(trace) = &entry.trace {
        let hill = resources.hills.hill(trace.meta.hill_idx).map_or_else(
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

fn load_replays(files: &FileStore) -> Vec<ReplayEntry> {
    let Ok(names) = files.list_by_ext_all("SJR") else {
        return Vec::new();
    };
    names
        .into_iter()
        .map(|filename| {
            let stem = if filename.len() > 4
                && filename.as_bytes()[filename.len() - 4..].eq_ignore_ascii_case(b".SJR")
            {
                filename[..filename.len() - 4].to_string()
            } else {
                filename.clone()
            };
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
