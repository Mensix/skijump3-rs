use engine::ui::{Element, Event, Key, View};
use crate::data::profile::Profile;
use crate::store::StoreRef;
use crate::route::RouteTarget;
use crate::palette_consts::*;

pub struct ProfilesView {
    store: StoreRef,
    selected: usize,
    editing: bool,
    edit_buf: String,
}

impl ProfilesView {
    pub fn new(store: StoreRef) -> Self {
        Self { store, selected: 0, editing: false, edit_buf: String::new() }
    }

    fn y_for(temp: usize) -> i32 {
        (temp * 8 + 4) as i32
    }

    fn col_y(temp: usize) -> i32 {
        match temp {
            0..=9 => (temp * 8 + 4) as i32,
            10..=15 => (temp * 8 + 10) as i32,
            _ => (temp * 16 - 118) as i32,
        }
    }
}

impl View<RouteTarget> for ProfilesView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        els.push(Element::fillbox(0, 0, 320, 200, 0));
        els.push(Element::fillbox(0, 0, 159, 200, BG_LEFT));
        els.push(Element::fillbox(160, 0, 160, 200, BG_RIGHT));
        els.push(Element::FillArea { thing: 63 });

        let np = self.store.borrow().profiles.num_profiles();
        if self.selected < np {
            els.push(Element::fillbox(166, 4, 154, 195, BG_RIGHT));
            els.push(Element::FillArea { thing: 63 });
        }

        els.push(Element::text_color("Jumpers:", 40, 3, FONT_HEADER));

        let store = self.store.borrow();
        let np = store.profiles.num_profiles();
        let has_slot = store.profiles.has_slot();
        let entries = if has_slot { np + 1 } else { np };

        for i in 0..np {
            let y = Self::y_for(i + 1);
            els.push(Element::fillbox(10, y - 1, 21, 8, BG_ORDER));
            let is_selected = self.selected == i && self.editing;
            if let Some(p) = store.profiles.profiles.get(i) {
                if is_selected {
                    els.push(Element::text_color(&self.edit_buf, 40, y, 240));
                    let cx = 40 + (self.edit_buf.len() as i32) * 7;
                    els.push(Element::fillbox(cx, y + 7, 2, 1, 240));
                } else {
                    els.push(Element::text_color(&p.name, 40, y, FONT_NAME));
                }
            }
        }

        if has_slot {
            let y = Self::y_for(np + 1);
            els.push(Element::text_color("*Create New Jumper*", 40, y, FONT_NEW));
        }

        let back_temp = if has_slot { np + 3 } else { np + 2 };
        els.push(Element::text_color("Back to Main Menu", 40, Self::y_for(back_temp), FONT_BACK));

        if np < 16 && self.selected < np {
            els.push(Element::fillbox(1, 175, 158, 25, BG_LEFT));
            els.push(Element::FillArea { thing: 63 });
            els.push(Element::text_color("(Use arrows,", 8, 175, 241));
            els.push(Element::text_color("ENTER edits jumper,", 11, 183, 241));
            els.push(Element::text_color("DEL removes from order)", 11, 191, 241));
        }

        if self.selected < np {
            if let Some(p) = store.profiles.profiles.get(self.selected) {
                let sx = 178;
                els.push(Element::fillbox(sx, 28, 19, 5, 216));
                els.push(Element::box_(sx, 28, 19, 5, 218));
                els.push(Element::fillbox(sx + 1, 37, 17, 3, 231));

                let labels: [&str; 18] = [
                    "Name:", "Real name:", "Suit Color:", "Ski Color:",
                    "Replace:", "Coach:", "Skip Quali:", "",
                    "", "Total Jumps:", "WC:", "Legs Won:",
                    "WC Won:", "Best:", "Best 4H:", "Longest WC:",
                    "Longest:", "KOTH:",
                ];

                for i in 0..18 {
                    let y = Self::col_y(i + 1);
                    if !labels[i].is_empty() {
                        els.push(Element::text_color(labels[i], 166, y, FONT_HEADER));
                    }
                    let val: &str = match i {
                        0 => &p.name,
                        1 => "",
                        2 | 3 => "",
                        4 => "-",
                        5 => "None",
                        6 => "Never",
                        7 | 8 => "",
                        9 => "0",
                        10 => "0",
                        11 => "0",
                        12 => "0",
                        13 => "-",
                        14 => "-",
                        15 => "-",
                        16 => "-",
                        17 => "-",
                        _ => "",
                    };
                    if !val.is_empty() {
                        let (vx, vy) = if i >= 15 {
                            (170, y + 8)
                        } else if labels[i].is_empty() {
                            (0, y)
                        } else {
                            let lw = store.font.string_width(labels[i]) as i32;
                            (166 + 4 + lw, y)
                        };
                        if vx > 0 {
                            els.push(Element::text_color(val, vx, vy, 246));
                        }
                    }
                }
            }
        }

        let (by, box_h) = if self.selected < entries {
            (10 + (self.selected as i32) * 8, 9)
        } else {
            (10 + (entries as i32) * 8, 9)
        };
        els.push(Element::box_(34, by, 123, box_h, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.editing {
            return match event {
                Event::Keyboard(Key::Enter) => {
                    if let Some(p) = self.store.borrow_mut().profiles.profiles.get_mut(self.selected) {
                        if !self.edit_buf.is_empty() {
                            p.name = self.edit_buf.clone();
                        }
                    }
                    self.editing = false;
                    self.edit_buf.clear();
                    None
                }
                Event::Keyboard(Key::Escape) => {
                    self.editing = false;
                    self.edit_buf.clear();
                    None
                }
                Event::Keyboard(Key::Up) | Event::Keyboard(Key::Down) => {
                    self.editing = false;
                    self.edit_buf.clear();
                    None
                }
                Event::Keyboard(Key::Char(c)) => {
                    if c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '.' {
                        self.edit_buf.push(c);
                    }
                    None
                }
                _ => None,
            };
        }

        let (np, has_slot) = {
            let store = self.store.borrow();
            (store.profiles.num_profiles(), store.profiles.has_slot())
        };
        let entries = if has_slot { np + 1 } else { np };
        let total = entries + 1;

        match event {
            Event::Keyboard(Key::Up) if self.selected > 0 => {
                self.selected -= 1;
                None
            }
            Event::Keyboard(Key::Up) => {
                self.selected = total - 1;
                None
            }
            Event::Keyboard(Key::Down) if self.selected + 1 < total => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selected = 0;
                None
            }
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                if self.selected < entries {
                    if has_slot && self.selected + 1 == entries {
                        let mut store = self.store.borrow_mut();
                        let new_idx = store.profiles.num_profiles();
                        let mut p = Profile::default();
                        p.name = "SKI JUMPER".to_string();
                        let mut counter: usize = 2;
                        loop {
                            let conflict = store.profiles.profiles.iter()
                                .any(|x| x.name == p.name);
                            if !conflict { break; }
                            p.name = format!("SKI JUMPER {}", counter);
                            counter += 1;
                            if counter > 200 { break; }
                        }
                        store.profiles.profiles.push(p);
                        store.profiles.edit_index = new_idx;
                        drop(store);
                        self.selected = new_idx;
                        self.edit_buf = self.store.borrow().profiles.profiles
                            .get(new_idx).map(|p| p.name.clone()).unwrap_or_default();
                        self.editing = true;
                        None
                    } else {
                        self.edit_buf = self.store.borrow().profiles.profiles
                            .get(self.selected).map(|p| p.name.clone()).unwrap_or_default();
                        self.editing = true;
                        None
                    }
                } else {
                    Some(RouteTarget::MainMenu)
                }
            }
            Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
            _ => None,
        }
    }
}
