use crate::competition::factory;
use crate::components::menu::{Menu, MenuItem};
use crate::components::screen;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Component, Element, Event, Key, View};
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KothMode {
    Main,
    Packs,
}

pub struct KothSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    menu: Menu,
    mode: Cell<KothMode>,
    pack_selection: Cell<usize>,
}

impl KothSetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let items = (0..6).map(|_| MenuItem::new(0, 0)).collect();
        let menu = Menu::new(11, 10, 160, 10, items, &resources.langbase, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false);
        let pack = {
            let cfg = resources.save_manager.config.borrow();
            cfg.kothpack
        };
        let pack_sel = if pack == 0 { 6 } else { (pack - 1) as usize };
        Self {
            resources,
            store,
            menu,
            mode: Cell::new(KothMode::Main),
            pack_selection: Cell::new(pack_sel.min(6)),
        }
    }

    fn config(&self) -> std::cell::Ref<'_, crate::save::config::Config> {
        self.resources.save_manager.config.borrow()
    }

    fn update_config(&self, f: impl FnOnce(&mut crate::save::config::Config)) {
        self.resources.save_manager.update_config(f);
    }

    fn start_koth(&self) -> Option<RouteTarget> {
        let profiles = self.store.profiles();
        let config = self.config();
        let hill_count = self.resources.hills.len();
        let comp = self.store.with_jump_rng_wind_mut(|rng, _| {
            factory::koth(&config, &profiles, self.resources.player_names(), hill_count, rng.clone())
        });
        drop(profiles);
        self.store.start_active(comp);
        Some(RouteTarget::CompetitionJump)
    }
}

impl View<RouteTarget> for KothSetupView {
    fn elements(&self) -> Vec<Element> {
        let mut els = screen::new_screen(2);
        let lang = &self.resources.langbase;
        let cfg = self.config();

        match self.mode.get() {
            KothMode::Main => {
                els.push(Element::text(&format!("1 - {}", lang.lstr(121)), 11, 10, FONT_DEFAULT, false));
                els.push(Element::text(&format!("2 - {}", lang.lstr(122)), 11, 20, FONT_DEFAULT, false));
                els.push(Element::text(&format!("3 - {}", lang.lstr(123)), 11, 30, FONT_DEFAULT, false));

                let hill_name = if cfg.kothmaki == 0 {
                    lang.lstr(9)
                } else {
                    self.resources.hills.hill(cfg.kothmaki as usize - 1)
                        .map(|h| h.name.as_str())
                        .unwrap_or("?")
                };
                els.push(Element::text(&format!("4 - {} {}", lang.lstr(124), hill_name), 11, 40, FONT_DEFAULT, false));

                let wind_str = if cfg.kothwind != 0 { lang.lstr(6) } else { lang.lstr(7) };
                els.push(Element::text(&format!("5 - {} {}", lang.lstr(125), wind_str), 11, 50, FONT_DEFAULT, false));

                let rounds_str = cfg.kothrounds.to_string();
                els.push(Element::text(&format!("6 - {} {}", lang.lstr(126), rounds_str), 11, 60, FONT_DEFAULT, false));

                els.push(Element::text(&format!("0 - {}", lang.lstr(127)), 11, 80, FONT_DEFAULT, false));

                els.push(Element::text(lang.lstr(120), 180, 10, FONT_DEFAULT, false));
                for i in 0..cfg.koth_count.min(20) as usize {
                    let idx = cfg.kothpel.get(i).copied().unwrap_or(1) as usize;
                    let name = self.resources.player_names()
                        .get(idx - 1)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    let y = 20 + (i + 1) * 8;
                    els.push(Element::text(&format!("{} #{}", name, idx), 180, y as i32, FONT_DEFAULT, false));
                }

                els.push(Element::text(lang.lstr(130), 11, 110, FONT_GOLD, false));
                for pack in 1..=7 {
                    let y = 120 + (pack - 1) * 8;
                    if pack == 7 {
                        let y7 = 168;
                        let title = koth_pack_title(pack, lang);
                        let is_selected = cfg.kothpack == 0;
                        els.push(Element::text(&title, 11, y7, if is_selected { FONT_GOLD } else { FONT_DEFAULT }, false));
                    } else {
                        let title = koth_pack_title(pack, lang);
                        let is_selected = cfg.kothpack == pack as i32;
                        els.push(Element::text(&title, 11, y as i32, if is_selected { FONT_GOLD } else { FONT_DEFAULT }, false));
                    }
                }
            }
            KothMode::Packs => {
                els.push(Element::text(lang.lstr(130), 11, 110, FONT_GOLD, false));
                let sel = self.pack_selection.get();
                for pack in 1..=7 {
                    let y = 120 + (pack - 1) * 8;
                    if pack == 7 {
                        let y7 = 168;
                        let title = koth_pack_title(pack, lang);
                        let is_sel = sel == 6;
                        els.push(Element::text(&title, 11, y7, if is_sel { FONT_GOLD } else { FONT_DEFAULT }, false));
                    } else {
                        let title = koth_pack_title(pack, lang);
                        let is_sel = sel == (pack - 1) as usize;
                        els.push(Element::text(&title, 11, y as i32, if is_sel { FONT_GOLD } else { FONT_DEFAULT }, false));
                    }
                }
                // box around selection
                let sy = if sel == 6 { 168i32 } else { 120 + sel as i32 * 8 };
                els.push(Element::box_(9, sy - 1, 202, 9, FONT_DEFAULT));
            }
        }

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.mode.get() {
            KothMode::Packs => match &event {
                Event::Keyboard(Key::Escape) => {
                    self.mode.set(KothMode::Main);
                    self.menu = Menu::new(11, 10, 160, 10, (0..6).map(|_| MenuItem::new(0, 0)).collect(), &self.resources.langbase, FONT_DEFAULT, FONT_DEFAULT)
                        .with_labels(false).with_box(false);
                    None
                }
                Event::Keyboard(Key::Up | Key::Left) => {
                    let s = self.pack_selection.get();
                    self.pack_selection.set(if s == 0 { 6 } else { s - 1 });
                    None
                }
                Event::Keyboard(Key::Down | Key::Right) => {
                    let s = self.pack_selection.get();
                    self.pack_selection.set(if s >= 6 { 0 } else { s + 1 });
                    None
                }
                Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                    let sel = self.pack_selection.get();
                    let new_pack = if sel == 6 { 0 } else { (sel + 1) as i32 };
                    self.update_config(|cfg| cfg.kothpack = new_pack);
                    // reload participants for new pack via getkoth equivalent
                    crate::competition::koth::builder::apply_koth_pack(
                        &self.resources.save_manager,
                        new_pack as u8,
                    );
                    self.mode.set(KothMode::Main);
                    self.menu = Menu::new(11, 10, 160, 10, (0..6).map(|_| MenuItem::new(0, 0)).collect(), &self.resources.langbase, FONT_DEFAULT, FONT_DEFAULT)
                        .with_labels(false).with_box(false);
                    None
                }
                _ => None,
            },
            KothMode::Main => {
                match &event {
                    Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
                    Event::Keyboard(Key::Char(ch)) if *ch >= '1' && *ch <= '6' => {
                        let n = *ch as usize - '0' as usize;
                        self.menu.set_selected(n - 1);
                        self.handle_menu_selection(n)
                    }
                    Event::Keyboard(Key::Char('0')) => Some(RouteTarget::MainMenu),
                    _ => {
                        self.menu.handle_event(&event).and_then(|idx| {
                            self.handle_menu_selection(idx + 1)
                        })
                    }
                }
            }
        }
    }
}

fn koth_pack_title(pack: u8, lang: &crate::text::lang::LangBase) -> String {
    match pack {
        1 => format!("1. {}", lang.lstr(131)),
        2 => format!("2. {}", lang.lstr(132)),
        3 => format!("3. {}", lang.lstr(133)),
        4 => format!("4. {}", lang.lstr(134)),
        5 => format!("5. {}", lang.lstr(135)),
        6 => format!("6. {}", lang.lstr(136)),
        7 => format!("0. {}", lang.lstr(137)),
        _ => format!("{pack}. ?"),
    }
}

impl KothSetupView {
    fn handle_menu_selection(&self, n: usize) -> Option<RouteTarget> {
        match n {
            1 => self.start_koth(),
            2 => {
                self.mode.set(KothMode::Packs);
                let cfg = self.config();
                let pack = cfg.kothpack;
                self.pack_selection.set(if pack == 0 { 6 } else { (pack - 1) as usize });
                None
            }
            3 => {
                // Choose Opponents - not yet implemented
                None
            }
            4 => {
                // Jumping at / select hill - not yet implemented
                None
            }
            5 => {
                self.update_config(|cfg| cfg.kothwind = if cfg.kothwind != 0 { 0 } else { 1 });
                None
            }
            6 => {
                self.update_config(|cfg| {
                    cfg.kothrounds = if cfg.kothrounds == 1 { 2 } else { 1 };
                });
                None
            }
            _ => None,
        }
    }
}
