use crate::components::screen;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD, FONT_HELP, FILL_DIM};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Element, Event, View};
use std::cell::Cell;

use crate::text::lang::LangBase;

pub struct KothSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    selected: Cell<usize>,
}

impl KothSetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        // Pascal getkoth: populate config with pack defaults
        let pack = resources.save_manager.config.borrow().kothpack;
        crate::competition::koth::builder::apply_koth_pack(
            &resources.save_manager,
            pack as u8,
        );
        Self {
            resources,
            store,
            selected: Cell::new(1),
        }
    }

    fn config(&self) -> std::cell::Ref<'_, crate::save::config::Config> {
        self.resources.save_manager.config.borrow()
    }

    fn col1(&self) -> engine::color::Rgba {
        let cfg = self.config();
        if cfg.kothpack > 0 { FONT_HELP } else { FONT_DEFAULT }
    }

    fn col2(&self) -> engine::color::Rgba {
        let cfg = self.config();
        if cfg.kothpack > 0 { FONT_HELP } else { FONT_GOLD }
    }
}

impl View<RouteTarget> for KothSetupView {
    fn elements(&self) -> Vec<Element> {
        let mut els = screen::new_screen(3);
        let lang = &self.resources.langbase;
        let cfg = self.config();

        // --- right panel: participant names (gold, Pascal 246) ---
        if cfg.koth_count > 0 {
            for i in 0..cfg.koth_count.min(20) as usize {
                let idx = cfg.kothpel.get(i).copied().unwrap_or(1) as usize;
                let name = self.resources.player_names()
                    .get(idx - 1)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                    let y = (20 + (i + 1) * 8) as i32;
                els.push(Element::text(&format!("{} #{}", name, idx), 180, y, FONT_GOLD, false));
            }
        } else {
            els.push(Element::text(lang.lstr(9), 180, 30, FONT_GOLD, false));
        }

        // --- right panel: "Computer Jumpers:" (white, Pascal 240) ---
        els.push(Element::text(lang.lstr(120), 180, 10, FONT_DEFAULT, false));

        // --- left panel: menu background (Pascal MakeMenu bgcolor=245) ---
        els.push(Element::fillbox(4, 7, 160, 63, FILL_DIM));
        els.push(Element::fill_area(63));

        // --- left panel: menu items ---
        // x=10, y starts at 10, item spacing 10

        // Item 1 - col0 (Pascal default fontcolor after fontcolor(240))
        els.push(Element::text(&format!("1 - {}", lang.lstr(121)), 10, 10, FONT_DEFAULT, false));
        // Item 2
        els.push(Element::text(&format!("2 - {}", lang.lstr(122)), 10, 20, FONT_DEFAULT, false));
        // Item 3 - col1
        els.push(Element::text(&format!("3 - {}", lang.lstr(123)), 10, 30, self.col1(), false));
        // Item 4 - col1 label + col2 value (Pascal hillname(0)=lstr(155) "Random WC Hill")
        els.push(Element::text(&format!("4 - {}", lang.lstr(124)), 10, 40, self.col1(), false));
        let hill_name = if cfg.kothmaki == 0 {
            lang.lstr(155)
        } else {
            self.resources.hills.hill(cfg.kothmaki as usize - 1)
                .map(|h| h.name.as_str())
                .unwrap_or("?")
        };
        els.push(Element::text(hill_name, 80, 40, self.col2(), false));
        // Item 5 - col1 label + col2 value
        els.push(Element::text(&format!("5 - {}", lang.lstr(125)), 10, 50, self.col1(), false));
        let wind_str = if cfg.kothwind != 0 { lang.lstr(6) } else { lang.lstr(7) };
        els.push(Element::text(wind_str, 80, 50, self.col2(), false));
        // Item 6 - col1 label + col2 value (Pascal lstr(kothrounds) = "One"/"Two")
        els.push(Element::text(&format!("6 - {}", lang.lstr(126)), 10, 60, self.col1(), false));
        els.push(Element::text(lang.lstr(cfg.kothrounds as usize), 80, 60, self.col2(), false));
        // Item 0 (white, Pascal 240)
        els.push(Element::text(&format!("0 - {}", lang.lstr(127)), 10, 80, FONT_DEFAULT, false));

        // --- left panel bottom: K.O.T.H Challenge Level (gold, Pascal 246) ---
        els.push(Element::text(lang.lstr(130), 10, 110, FONT_GOLD, false));

        // --- left panel bottom: pack list (Pascal kothchallenge) ---
        let selected_pack = if cfg.kothpack == 0 { 7u8 } else { cfg.kothpack as u8 };
        let mut py = 120i32;
        for pack in 1..=7u8 {
            let title = koth_pack_title(pack, lang);
            let color = if selected_pack == pack { FONT_DEFAULT } else { FONT_HELP };
            els.push(Element::text(&title, 10, py, color, false));
            py += 8;
            if pack == 6 { py += 8; }
        }

        // selection box around current menu item (Pascal MakeMenu)
        let sel = self.selected.get();
        if sel >= 1 && sel <= 6 {
            let sy = (10 + (sel - 1) * 10) as i32;
            els.push(Element::box_(4, sy - 3, 160, 10, FONT_DEFAULT));
        }

        els
    }

    fn handle_event(&mut self, _event: Event) -> Option<RouteTarget> {
        None
    }
}

fn koth_pack_title(pack: u8, lang: &LangBase) -> String {
    match pack {
        1 => format!("0. {}", lang.lstr(131)),
        2 => format!("1. {}", lang.lstr(132)),
        3 => format!("2. {}", lang.lstr(133)),
        4 => format!("3. {}", lang.lstr(134)),
        5 => format!("4. {}", lang.lstr(135)),
        6 => format!("5. {}", lang.lstr(136)),
        7 => format!("6. {}", lang.lstr(137)),
        _ => format!("{pack}. ?"),
    }
}
