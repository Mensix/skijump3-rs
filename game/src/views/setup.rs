use crate::components::menu::{Menu, MenuItem};
use crate::components::screen;
use crate::gfx::palette::{BG_LEFT, FILL_BORDER, FILL_DIM, FONT_DEFAULT, FONT_HEADER, FONT_HELP};
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveManager;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::lang::LangBase;
use engine::ui::{Element, Event, Key, View};
use std::cell::Cell;
use std::rc::Rc;

/// Pascal: setupmenu (SJ3.PAS:494-737).
/// 1:1 port of the Setup / Game Settings menu with screens:
///   0=Main, 1=General, 2=Jumping, 3=Hiscore.
pub struct SetupView {
    resources: ResourcesRef,
    store: StoreRef,
    /// Current screen index (0..3).
    screen: Cell<usize>,
    /// Menu selection tracker (labels+box disabled, per-screen).
    menu: Menu,
    modal: Cell<Option<SetupModal>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SetupModal {
    /// Pascal choosewindplace: pick windmeter position.
    WindPlace(usize),
    /// Pascal chooseseecomps: pick number/name of computer opponents to view.
    SeeComps(usize),
    /// Confirm then ResetHiscore(kind). kind=1 reset, kind=0 zero.
    ConfirmReset(u8),
    /// Language picker.
    LanguagePicker(usize),
}

/// Pascal: `HexCh`[0..15] = '0123456789ABCDEF' (0-indexed)
fn hex_char(index: usize) -> &'static str {
    match index {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "A",
        11 => "B",
        12 => "C",
        13 => "D",
        14 => "E",
        15 => "F",
        _ => "?",
    }
}

/// Pascal: WindPlaceName(place) — combos of lstr(390..396).
fn wind_place_name(langbase: &LangBase, place: usize) -> String {
    match place {
        1 => format!("{}-{}", langbase.lstr(392), langbase.lstr(393)),
        2 => format!("{}-{}", langbase.lstr(391), langbase.lstr(393)),
        3 => format!("{}-{}", langbase.lstr(392), langbase.lstr(395)),
        4 => format!("{}-{}", langbase.lstr(392), langbase.lstr(394)),
        5 => format!("{}-{}", langbase.lstr(391), langbase.lstr(395)),
        6 => format!("{}-{}", langbase.lstr(390), langbase.lstr(395)),
        7 => format!("{}-{}", langbase.lstr(390), langbase.lstr(394)),
        8 => format!("{}-{}", langbase.lstr(390), langbase.lstr(393)),
        11 => format!("{}: {}", langbase.lstr(396), langbase.lstr(390)),
        12 => format!("{}: {}", langbase.lstr(396), langbase.lstr(391)),
        13 => format!("{}: {}", langbase.lstr(396), langbase.lstr(392)),
        _ => unreachable!(),
    }
}

impl SetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let menu = Self::make_menu(0, &resources.langbase, 0);
        Self {
            resources,
            store,
            screen: Cell::new(0),
            menu,
            modal: Cell::new(None),
        }
    }

    fn make_menu(screen: usize, langbase: &Rc<LangBase>, selected: usize) -> Menu {
        let entries = match screen {
            0 => 6,
            1 => 4,
            2 => 11,
            3 => 5,
            _ => 0,
        };
        let items = (0..entries).map(|_| MenuItem::new(0, 0)).collect();
        let mut m = Menu::new(35, 40, 221, 10, items, langbase, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .with_exit(154, 0);
        m.set_selected(selected.min(entries));
        m
    }

    fn switch_screen(&mut self, new_screen: usize) {
        let selected = self.menu.selected();
        self.screen.set(new_screen);
        self.menu = Self::make_menu(new_screen, &self.resources.langbase, selected);
    }

    fn langbase(&self) -> &LangBase {
        &self.resources.langbase
    }

    fn config(&self) -> std::cell::Ref<'_, Config> {
        self.resources.save_manager.config.borrow()
    }

    fn save_manager(&self) -> &SaveManager {
        &self.resources.save_manager
    }

    fn rect_bg(x: i32, y: i32, w: i32, h: i32) -> Vec<Element> {
        vec![
            Element::fillbox(x, y, w, h, FILL_BORDER),
            Element::fillbox(x + 1, y + 1, w - 2, h - 2, BG_LEFT),
        ]
    }

    /// Pascal: SetupItem(temp, screen, entries, str1) — render one row.
    /// index=0 is the exit row.
    fn setup_item(&self, els: &mut Vec<Element>, index: usize, entries: usize, value_str: &str) {
        let xx = 25;
        let yy = if index == 0 {
            (entries as i32) * 10 + 50
        } else {
            (index as i32) * 10 + 30
        };

        let row_label = format!("{}.", hex_char(index));

        // Row number (Pascal: ewritefont at xx=25, right-aligned)
        els.push(Element::text(row_label, xx, yy, FONT_HEADER, true));

        // Label text (Pascal: writefont at xx=35)
        let label_id = match (self.screen.get(), index) {
            (0, 0) => 195,
            (0, 1) => 196,
            (0, 2) => 197,
            (0, 3) => 198,
            (0, 4) => 199,
            (0, 5) => 200,
            (0, 6) => 201,
            (1, 0) => 203,
            (1, 1) => 204,
            (1, 2) => 205,
            (1, 3) => 206,
            (1, 4) => 207,
            (2, 0) => 211,
            (2, 1) => 212,
            (2, 2) => 213,
            (2, 3) => 214,
            (2, 4) => 215,
            (2, 5) => 216,
            (2, 6) => 217,
            (2, 7) => 218,
            (2, 8) => 219,
            (2, 9) => 220,
            (2, 10) => 221,
            (2, 11) => 222,
            (3, 0) => 225,
            (3, 1) => 226,
            (3, 2) => 227,
            (3, 3) => 228,
            (3, 4) => 229,
            (3, 5) => 230,
            _ => return,
        };

        els.push(Element::text(
            self.langbase().lstr(label_id),
            35,
            yy,
            FONT_DEFAULT,
            false,
        ));

        // Right-side value (Pascal: writefont at xx=255)
        if !value_str.is_empty() {
            els.push(Element::text(value_str, 255, yy, FONT_HEADER, false));
        }
    }

    fn render_screen(&self, els: &mut Vec<Element>) {
        // Pascal: NewScreen(1,0) — SJ3GRAPH.PAS:81-187
        // style 1: fills + fillarea(63) pattern + logo sprite
        els.extend(screen::new_screen(1));

        // Title (Pascal: writefont(30,6,str1))
        let title_id = match self.screen.get() {
            0 => 175,
            1 => 176,
            2 => 177,
            3 => 178,
            _ => return,
        };
        els.push(Element::text(
            self.langbase().lstr(title_id),
            30,
            6,
            FONT_DEFAULT,
            false,
        ));

        let cfg = self.config();
        let screen = self.screen.get();
        let entries = self.menu.item_count();

        // Pascal: for temp:=0 to entries do SetupItem(...)
        // Rust: temp=0 is exit row (no value), temps 1..=entries are items.
        // Value match uses 0-indexed (temp-1) for clean 0-based arms.
        for temp in 0..=entries {
            let value_str = if temp > 0 {
                match (screen, temp - 1) {
                    (1, 0) => {
                        let ln = cfg.languagenumber;
                        let all = &self.langbase().languages;
                        if ln >= 0 && (ln as usize) < all.len() {
                            all[ln as usize].clone()
                        } else {
                            "Undecided".to_string()
                        }
                    }
                    (1, 1) => {
                        if cfg.beeppi != 0 {
                            self.langbase().lstr(6).to_string()
                        } else {
                            self.langbase().lstr(7).to_string()
                        }
                    }
                    (1, 2) => {
                        if cfg.gdetail == 0 {
                            self.langbase().lstr(13).to_string()
                        } else {
                            self.langbase().lstr(14).to_string()
                        }
                    }
                    (1, 3) => {
                        let n = cfg.namenumber;
                        let hint = self.resources.namesets.title_for_config(n);
                        els.push(Element::text(hint.to_string(), 40, 78, FONT_HELP, false));
                        format!("{n}")
                    }
                    (2, 0) => {
                        if cfg.trainrounds == 0 {
                            self.langbase().lstr(9).to_string()
                        } else {
                            format!("{}", cfg.trainrounds)
                        }
                    }
                    (2, 1) => {
                        if cfg.lct != 0 {
                            self.langbase().lstr(180).to_string()
                        } else {
                            self.langbase().lstr(185).to_string()
                        }
                    }
                    (2, 2) => {
                        if cfg.diff != 0 {
                            self.langbase().lstr(181).to_string()
                        } else {
                            self.langbase().lstr(186).to_string()
                        }
                    }
                    (2, 3) => {
                        if cfg.diffwc != 0 {
                            self.langbase().lstr(181).to_string()
                        } else {
                            self.langbase().lstr(186).to_string()
                        }
                    }
                    (2, 4) => {
                        if cfg.compactlist != 0 {
                            self.langbase().lstr(182).to_string()
                        } else {
                            self.langbase().lstr(187).to_string()
                        }
                    }
                    (2, 5) => {
                        if cfg.invback != 0 {
                            self.langbase().lstr(183).to_string()
                        } else {
                            self.langbase().lstr(188).to_string()
                        }
                    }
                    (2, 6) => {
                        if cfg.automatichrr != 0 {
                            self.langbase().lstr(182).to_string()
                        } else {
                            self.langbase().lstr(185).to_string()
                        }
                    }
                    (2, 7) => {
                        if cfg.goals != 0 {
                            self.langbase().lstr(180).to_string()
                        } else {
                            self.langbase().lstr(186).to_string()
                        }
                    }
                    (2, 8) => {
                        if cfg.seecomps > 240 {
                            self.langbase().lstr(cfg.seecomps as usize).to_string()
                        } else {
                            format!("#{}", cfg.seecomps)
                        }
                    }
                    (2, 9) => wind_place_name(self.langbase(), cfg.windplace as usize),
                    (2, 10) => {
                        if cfg.kosystem != 0 {
                            self.langbase().lstr(182).to_string()
                        } else {
                            self.langbase().lstr(185).to_string()
                        }
                    }
                    (3, 0) => {
                        if cfg.comphrs != 0 {
                            self.langbase().lstr(183).to_string()
                        } else {
                            self.langbase().lstr(187).to_string()
                        }
                    }
                    (3, 1) => {
                        if cfg.nosamename != 0 {
                            self.langbase().lstr(185).to_string()
                        } else {
                            self.langbase().lstr(180).to_string()
                        }
                    }
                    _ => String::new(),
                }
            } else {
                String::new()
            };
            self.setup_item(els, temp, entries, &value_str);
        }

        // Selection box (Pascal: MakeMenu — box + fillarea at cursor)
        let sel = self.menu.selected();
        if sel <= entries {
            let by = if sel < entries {
                40 - 3 + (sel as i32) * 10
            } else {
                (entries as i32) * 10 + 50 - 3
            };
            els.push(Element::box_(35 - 6, by, 221 + 1, 10 + 1, FONT_DEFAULT));
        }
    }

    fn handle_screen_event(&mut self, event: Event) -> Option<RouteTarget> {
        let screen = self.screen.get();
        let entries = self.menu.item_count();

        match event {
            Event::Keyboard(Key::Up) => {
                let sel = self.menu.selected();
                let new_sel = if sel == 0 { entries } else { sel - 1 };
                self.menu.set_selected(new_sel);
            }
            Event::Keyboard(Key::Down) => {
                let sel = self.menu.selected();
                let new_sel = if sel >= entries { 0 } else { sel + 1 };
                self.menu.set_selected(new_sel);
            }
            Event::Keyboard(Key::Escape) => {
                if screen == 0 {
                    return Some(RouteTarget::MainMenu);
                }
                self.switch_screen(0);
            }
            Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                let sel = self.menu.selected();
                if sel >= entries {
                    if screen == 0 {
                        return Some(RouteTarget::MainMenu);
                    }
                    self.switch_screen(0);
                } else {
                    self.activate_item(screen, sel);
                }
            }
            Event::Keyboard(Key::Char(c)) if c.is_ascii_digit() => {
                if let Some(d) = c.to_digit(10) {
                    let n = d as usize;
                    if n >= 1 && n <= entries {
                        self.menu.set_selected(n - 1);
                    } else if n == 0 {
                        self.menu.set_selected(entries);
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn activate_item(&mut self, screen: usize, item: usize) {
        match (screen, item) {
            (0, 0..=2) => {
                self.switch_screen(item + 1);
            }
            // (0, 3..=5) handled by wildcard — configurekeys, setgoals, hillmaker
            (1, 0) => {
                let current = self.config().languagenumber;
                let idx = if current >= 0 { current as usize } else { 0 };
                let langs = &self.langbase().languages;
                let idx = idx.min(langs.len().saturating_sub(1));
                self.modal.set(Some(SetupModal::LanguagePicker(idx)));
            }
            (1, 1) => self
                .save_manager()
                .update_config(|cfg| cfg.beeppi = i32::from(cfg.beeppi == 0)),
            (1, 2) => self
                .save_manager()
                .update_config(|cfg| cfg.gdetail = i32::from(cfg.gdetail == 0)),
            (1, 3) => {
                let ns_len = self.resources.namesets.len();
                self.save_manager()
                    .update_config(|cfg| cfg.namenumber = (cfg.namenumber + 1) % ns_len as i32);
            }
            (2, 0) => self
                .save_manager()
                .update_config(|cfg| cfg.trainrounds = (cfg.trainrounds + 1) % 4),
            (2, 1) => self
                .save_manager()
                .update_config(|cfg| cfg.lct = i32::from(cfg.lct == 0)),
            (2, 2) => self
                .save_manager()
                .update_config(|cfg| cfg.diff = i32::from(cfg.diff == 0)),
            (2, 3) => self
                .save_manager()
                .update_config(|cfg| cfg.diffwc = i32::from(cfg.diffwc == 0)),
            (2, 4) => self
                .save_manager()
                .update_config(|cfg| cfg.compactlist = i32::from(cfg.compactlist == 0)),
            (2, 5) => self
                .save_manager()
                .update_config(|cfg| cfg.invback = i32::from(cfg.invback == 0)),
            (2, 6) => self
                .save_manager()
                .update_config(|cfg| cfg.automatichrr = i32::from(cfg.automatichrr == 0)),
            (2, 7) => self
                .save_manager()
                .update_config(|cfg| cfg.goals = i32::from(cfg.goals == 0)),
            (2, 8) => {
                let current = self.config().seecomps;
                let idx = if current >= 1 { current as usize } else { 240 };
                self.modal.set(Some(SetupModal::SeeComps(idx)));
            }
            (2, 9) => {
                let place = self.config().windplace;
                let pos = if place <= 8 { place - 1 } else { place - 3 };
                self.modal.set(Some(SetupModal::WindPlace(pos as usize)));
            }
            (2, 10) => self
                .save_manager()
                .update_config(|cfg| cfg.kosystem = i32::from(cfg.kosystem == 0)),
            (3, 0) => self
                .save_manager()
                .update_config(|cfg| cfg.comphrs = i32::from(cfg.comphrs == 0)),
            (3, 1) => self
                .save_manager()
                .update_config(|cfg| cfg.nosamename = i32::from(cfg.nosamename == 0)),
            (3, 2) => self.modal.set(Some(SetupModal::ConfirmReset(1))),
            (3, 3) => self.modal.set(Some(SetupModal::ConfirmReset(0))),
            (3, 4) => self.reset_config_defaults(),
            _ => {}
        }
    }

    fn reset_config_defaults(&self) {
        self.save_manager().update_config(|cfg| {
            *cfg = Config::default();
        });
    }
}

impl View<RouteTarget> for SetupView {
    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::new();
        self.render_screen(&mut els);

        match self.modal.get() {
            Some(SetupModal::WindPlace(pos)) => {
                // Pascal: choosewindplace (SJ3UNIT.PAS:2240-2281)
                els.extend(Self::rect_bg(54, 19, 222, 162));
                els.push(Element::text(
                    self.langbase().lstr(221),
                    75,
                    30,
                    FONT_HEADER,
                    false,
                ));

                let winds = 11;
                for apu1 in 1..=winds {
                    let yy = (apu1 as i32) * 10 + 34;
                    let name = if apu1 <= 8 {
                        wind_place_name(self.langbase(), apu1)
                    } else {
                        wind_place_name(self.langbase(), apu1 + 2)
                    };
                    els.push(Element::text(format!("{apu1}."), 85, yy, FONT_HEADER, true));
                    let color = if (apu1 - 1) == pos {
                        FONT_HEADER
                    } else {
                        FONT_DEFAULT
                    };
                    els.push(Element::text(name, 90, yy, color, false));
                }

                // Row 0 = exit
                let yy = (winds * 10 + 34 + 20) as i32;
                els.push(Element::text(
                    format!("0.{}", self.langbase().lstr(154)),
                    85,
                    yy,
                    FONT_DEFAULT,
                    false,
                ));
                // Help
                els.push(Element::text(
                    self.langbase().lstr(150),
                    75,
                    175,
                    FONT_HELP,
                    false,
                ));
            }
            Some(SetupModal::SeeComps(val)) => {
                // Pascal: chooseseecomps (SJ3INFO.PAS:1806-1872)
                els.extend(Self::rect_bg(74, 79, 172, 54));
                els.push(Element::text(
                    self.langbase().lstr(220),
                    85,
                    85,
                    FONT_DEFAULT,
                    false,
                ));
                els.push(Element::text(
                    self.langbase().lstr(150),
                    85,
                    95,
                    FONT_HELP,
                    false,
                ));
                let display = if val > 240 {
                    self.langbase().lstr(val).to_string()
                } else {
                    format!("#{val}")
                };
                els.push(Element::fillbox(85, 105, 150, 20, FILL_DIM));
                els.push(Element::text(display, 95, 112, FONT_HEADER, false));
            }
            Some(SetupModal::ConfirmReset(kind)) => {
                // Pascal: fillbox + text + Y/N
                els.extend(Self::rect_bg(69, 79, 182, 52));
                let label = if kind == 1 {
                    self.langbase().lstr(190)
                } else {
                    self.langbase().lstr(191)
                };
                els.push(Element::text(
                    format!("{} {}", label, self.langbase().lstr(192)),
                    80,
                    90,
                    FONT_DEFAULT,
                    false,
                ));
                els.push(Element::text(
                    self.langbase().lstr(193),
                    80,
                    110,
                    FONT_DEFAULT,
                    false,
                ));
            }
            Some(SetupModal::LanguagePicker(sel)) => {
                let langs = &self.langbase().languages;
                // Pascal: non-full WelcomeScreen — fillbox(74,41,246,186,248)
                els.push(Element::fillbox(74, 41, 173, 146, FILL_BORDER));
                els.push(Element::fillbox(75, 42, 171, 144, BG_LEFT));
                // Title: writefont(100,50,'PLEASE CHOOSE A LANGUAGE:')
                els.push(Element::text(
                    "PLEASE CHOOSE A LANGUAGE:",
                    100,
                    50,
                    FONT_DEFAULT,
                    false,
                ));
                // Language names: Pascal temp:=1..numlanguages, y=temp*8+55
                for (i, name) in langs.iter().enumerate() {
                    let yy = ((i + 1) as i32) * 8 + 55;
                    els.push(Element::center_text(name, 155, yy, FONT_HEADER));
                }
                // Selection box (Pascal MakeMenu phase=7: box outline, no fillarea)
                let bx = 112 - 6;
                let by = 64 - 3 + (sel as i32) * 8;
                els.push(Element::box_(bx, by, 100 + 1, 8 + 1, FONT_DEFAULT));
            }
            None => {}
        }

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.modal.get() {
            Some(SetupModal::WindPlace(pos)) => {
                let winds = 11;
                match event {
                    Event::Keyboard(Key::Up) => {
                        let new_pos = if pos == 0 { winds - 1 } else { pos - 1 };
                        self.modal.set(Some(SetupModal::WindPlace(new_pos)));
                    }
                    Event::Keyboard(Key::Down) => {
                        let new_pos = if pos >= winds - 1 { 0 } else { pos + 1 };
                        self.modal.set(Some(SetupModal::WindPlace(new_pos)));
                    }
                    Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                        let place = if pos < 8 { pos + 1 } else { pos + 3 };
                        self.save_manager()
                            .update_config(|cfg| cfg.windplace = place as i32);
                        self.store.jump_runtime.set_wind_place(place as u8);
                        self.modal.set(None);
                    }
                    Event::Keyboard(Key::Escape) => {
                        self.modal.set(None);
                    }
                    _ => {}
                }
                None
            }
            Some(SetupModal::SeeComps(mut val)) => {
                let num_players: i32 = 250;
                match event {
                    Event::Keyboard(Key::Up | Key::Left) => {
                        if val > 1 {
                            val -= 1;
                        } else {
                            val = 240;
                        }
                        if val < 235 && val > num_players as usize {
                            val = (num_players as usize).saturating_sub(11);
                        }
                        self.modal.set(Some(SetupModal::SeeComps(val)));
                    }
                    Event::Keyboard(Key::Down | Key::Right) => {
                        if val >= 240 {
                            val = 1;
                        } else {
                            val += 1;
                        }
                        if val > num_players as usize - 11 && val < 235 {
                            val = 235;
                        }
                        self.modal.set(Some(SetupModal::SeeComps(val)));
                    }
                    Event::Keyboard(Key::Enter) => {
                        self.save_manager()
                            .update_config(|cfg| cfg.seecomps = val as i32);
                        self.modal.set(None);
                    }
                    Event::Keyboard(Key::Escape) => {
                        self.modal.set(None);
                    }
                    _ => {}
                }
                None
            }
            Some(SetupModal::ConfirmReset(kind)) => {
                match event {
                    Event::Keyboard(Key::Char(c)) if c == 'y' || c == 'Y' => {
                        // Pascal: ResetHiscore(kind) where kind=1=reset, 0=zero
                        let _ = kind;
                        self.modal.set(None);
                    }
                    Event::Keyboard(Key::Escape | Key::Enter | Key::Char('n' | 'N')) => {
                        self.modal.set(None);
                    }
                    _ => {}
                }
                None
            }
            Some(SetupModal::LanguagePicker(sel)) => {
                let langs = &self.langbase().languages;
                match event {
                    Event::Keyboard(Key::Up) => {
                        let new_sel = if sel == 0 { langs.len() - 1 } else { sel - 1 };
                        self.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
                    }
                    Event::Keyboard(Key::Down) => {
                        let new_sel = if sel >= langs.len() - 1 { 0 } else { sel + 1 };
                        self.modal.set(Some(SetupModal::LanguagePicker(new_sel)));
                    }
                    Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                        self.save_manager().set_language(sel);
                        self.modal.set(None);
                    }
                    Event::Keyboard(Key::Escape) => {
                        self.modal.set(None);
                    }
                    _ => {}
                }
                None
            }
            None => self.handle_screen_event(event),
        }
    }
}
