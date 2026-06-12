use crate::competition::factory;
use crate::gfx::palette::{BG_LEFT, BG_RIGHT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::layout::shorten_name;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Event, Key};
use std::cell::Cell;

use crate::text::lang::LangBase;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KothMode {
    Main,
    Packs,
}

pub struct KothSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    selected: Cell<usize>,
    mode: Cell<KothMode>,
    pack_cursor: Cell<usize>,
}

impl KothSetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let pack = resources.save_manager.config.borrow().kothpack;
        crate::competition::koth::builder::apply_koth_pack(&resources.save_manager, pack as u8);
        Self {
            resources,
            store,
            selected: Cell::new(1),
            mode: Cell::new(KothMode::Main),
            pack_cursor: Cell::new(0),
        }
    }

    fn config(&self) -> std::cell::Ref<'_, crate::save::config::Config> {
        self.resources.save_manager.config.borrow()
    }

    fn update_config(&self, f: impl FnOnce(&mut crate::save::config::Config)) {
        self.resources.save_manager.update_config(f);
    }

    fn col1(&self) -> engine::color::Rgba {
        let cfg = self.config();
        if cfg.kothpack > 0 {
            FONT_HELP
        } else {
            FONT_DEFAULT
        }
    }

    fn col2(&self) -> engine::color::Rgba {
        let cfg = self.config();
        if cfg.kothpack > 0 {
            FONT_HELP
        } else {
            FONT_GOLD
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.fill((0, 0, 169, 99), FILL_DIM);
        cx.fill((0, 100, 169, 100), BG_RIGHT);
        cx.fill((170, 0, 150, 200), BG_LEFT);
        cx.dither_fill(63);
        let lang = &self.resources.langbase;
        let cfg = self.config();

        // --- right panel: participant names (gold, Pascal 246) ---
        if cfg.koth_count > 0 {
            for i in 0..cfg.koth_count.min(20) as usize {
                let idx = cfg.kothpel.get(i).copied().unwrap_or(1) as usize;
                let name = self
                    .resources
                    .player_names()
                    .get(idx - 1)
                    .map(|s| shorten_name(s, &self.resources.font, 110))
                    .unwrap_or_else(|| "?".to_string());
                let y = (20 + (i + 1) * 8) as i32;
                cx.text((180, y), FONT_GOLD, format!("{} #{}", name, idx));
            }
        } else {
            cx.text((180, 30), FONT_GOLD, lang.lstr(9));
        }

        // --- right panel: "Computer Jumpers:" (white, Pascal 240) ---
        cx.text((180, 10), FONT_DEFAULT, lang.lstr(120));

        // --- left panel: menu background (Pascal MakeMenu bgcolor=245) ---
        cx.fill((4, 7, 160, 63), FILL_DIM);
        cx.dither_fill(63);

        // --- left panel: menu items ---
        cx.text((10, 10), FONT_DEFAULT, format!("1 - {}", lang.lstr(121)));
        cx.text((10, 20), FONT_DEFAULT, format!("2 - {}", lang.lstr(122)));
        cx.text((10, 30), self.col1(), format!("3 - {}", lang.lstr(123)));
        cx.text((10, 40), self.col1(), format!("4 - {}", lang.lstr(124)));
        let hill_name = if cfg.kothmaki == 0 {
            lang.lstr(155)
        } else {
            self.resources
                .hills
                .hill(cfg.kothmaki as usize - 1)
                .map(|h| h.name.as_str())
                .unwrap_or("?")
        };
        cx.text((80, 40), self.col2(), hill_name);
        cx.text((10, 50), self.col1(), format!("5 - {}", lang.lstr(125)));
        let wind_str = if cfg.kothwind != 0 {
            lang.lstr(6)
        } else {
            lang.lstr(7)
        };
        cx.text((80, 50), self.col2(), wind_str);
        cx.text((10, 60), self.col1(), format!("6 - {}", lang.lstr(126)));
        cx.text((80, 60), self.col2(), lang.lstr(cfg.kothrounds as usize));
        cx.text((10, 80), FONT_DEFAULT, format!("0 - {}", lang.lstr(127)));

        // --- left panel bottom: K.O.T.H Challenge Level (gold, Pascal 246) ---
        cx.text((10, 110), FONT_GOLD, lang.lstr(130));

        // --- left panel bottom: pack list (Pascal kothchallenge) ---
        let is_pack_mode = self.mode.get() == KothMode::Packs;
        let mut py = 120i32;
        for pack in 1..=7u8 {
            let title = koth_pack_title(pack, lang);
            // In pack mode all items are white (Pascal kothchallenge(x,255))
            let color = if is_pack_mode {
                FONT_DEFAULT
            } else {
                let selected_pack = if cfg.kothpack == 0 {
                    7u8
                } else {
                    cfg.kothpack as u8
                };
                if selected_pack == pack {
                    FONT_DEFAULT
                } else {
                    FONT_HELP
                }
            };
            cx.text((10, py), color, title);
            py += 8;
            if pack == 6 {
                py += 8;
            }
        }

        match self.mode.get() {
            KothMode::Main => {
                let sel = self.selected.get();
                if sel >= 1 && sel <= 6 {
                    let sy = (10 + (sel - 1) * 10) as i32;
                    cx.stroke((4, sy - 3, 160, 10), FONT_DEFAULT);
                }
            }
            KothMode::Packs => {
                let cur = self.pack_cursor.get();
                let pcy = pack_cursor_y(cur);
                cx.stroke((4, pcy - 2, 160, 8), FONT_DEFAULT);
            }
        }
    }

    fn handle_input(&mut self, event: Event) -> Option<RouteTarget> {
        match self.mode.get() {
            KothMode::Main => self.handle_main(event),
            KothMode::Packs => self.handle_packs(event),
        }
    }
}

impl Screen<RouteTarget> for KothSetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
        if let Some(route) = self.handle_input(event) {
            cx.navigate(route);
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn input_from_ui(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(c) => Some(Event::Keyboard(Key::Char(c))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

impl KothSetupView {
    fn handle_main(&mut self, event: Event) -> Option<RouteTarget> {
        match &event {
            Event::Keyboard(Key::Escape) => return Some(RouteTarget::MainMenu),
            Event::Keyboard(Key::Up | Key::Left) => {
                let s = self.selected.get();
                self.selected.set(if s <= 1 { 6 } else { s - 1 });
                return None;
            }
            Event::Keyboard(Key::Down | Key::Right) => {
                let s = self.selected.get();
                self.selected.set(if s >= 6 { 1 } else { s + 1 });
                return None;
            }
            Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                return self.activate(self.selected.get());
            }
            Event::Keyboard(Key::Char(ch)) if *ch >= '1' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.selected.set(n);
                return self.activate(n);
            }
            Event::Keyboard(Key::Char('0')) => return Some(RouteTarget::MainMenu),
            _ => {}
        }
        None
    }

    fn handle_packs(&mut self, event: Event) -> Option<RouteTarget> {
        match &event {
            Event::Keyboard(Key::Escape) => {
                self.mode.set(KothMode::Main);
                None
            }
            Event::Keyboard(Key::Up | Key::Left) => {
                let c = self.pack_cursor.get();
                self.pack_cursor.set(if c == 0 { 6 } else { c - 1 });
                None
            }
            Event::Keyboard(Key::Down | Key::Right) => {
                let c = self.pack_cursor.get();
                self.pack_cursor.set(if c >= 6 { 0 } else { c + 1 });
                None
            }
            Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                let cur = self.pack_cursor.get();
                let pack = if cur == 0 { 0 } else { cur as i32 };
                self.apply_pack(pack);
                None
            }
            Event::Keyboard(Key::Char(ch)) if *ch >= '1' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.pack_cursor.set(n);
                None
            }
            Event::Keyboard(Key::Char('0')) => {
                self.pack_cursor.set(0);
                None
            }
            _ => None,
        }
    }

    fn apply_pack(&self, pack: i32) {
        self.update_config(|cfg| cfg.kothpack = pack);
        crate::competition::koth::builder::apply_koth_pack(
            &self.resources.save_manager,
            pack as u8,
        );
        self.mode.set(KothMode::Main);
    }

    fn activate(&self, n: usize) -> Option<RouteTarget> {
        match n {
            1 => self.start_koth(),
            2 => {
                let cfg = self.config();
                let pack = cfg.kothpack;
                self.pack_cursor
                    .set(if pack == 0 { 0 } else { pack as usize });
                self.mode.set(KothMode::Packs);
                None
            }
            3 | 4 => None,
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

    fn start_koth(&self) -> Option<RouteTarget> {
        let profiles = self.store.profiles();
        let config = self.config();
        let hill_count = self.resources.hills.len();
        let comp = self.store.with_jump_rng_wind_mut(|rng, _| {
            factory::koth(
                &config,
                &profiles,
                self.resources.player_names(),
                hill_count,
                rng.clone(),
            )
        });
        drop(profiles);
        self.store.start_active(comp);
        Some(RouteTarget::CompetitionJump)
    }
}

fn pack_cursor_y(cur: usize) -> i32 {
    if cur == 0 {
        176
    } else {
        120 + (cur as i32 - 1) * 8
    }
}

fn koth_pack_title(pack: u8, lang: &LangBase) -> String {
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
