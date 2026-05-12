use engine::ui::{Element, Event, Key, View};
use crate::data::profile::Profile;
use crate::store::StoreRef;
use crate::route::RouteTarget;

const BG_LEFT: u8 = 243;
const BG_RIGHT: u8 = 244;
const FONT_HEADER: u8 = 241;
const FONT_NAME: u8 = 240;
const FONT_NEW: u8 = 246;
const FONT_BACK: u8 = 240;
const BG_ORDER: u8 = 243;

pub struct ProfilesView {
    store: StoreRef,
    selected: usize,
    editing: bool,
    edit_buf: String,
}

impl ProfilesView {
    pub fn new(store: StoreRef) -> Self {
        Self { store, selected: 1, editing: false, edit_buf: String::new() }
    }

    fn y_for(temp: usize) -> i32 {
        (temp * 8 + 4) as i32
    }
}

impl View<RouteTarget> for ProfilesView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        els.push(Element::fillbox(0, 0, 320, 200, 0));
        els.push(Element::fillbox(0, 0, 159, 200, BG_LEFT));
        els.push(Element::fillbox(160, 0, 160, 200, BG_RIGHT));
        els.push(Element::FillArea { thing: 63 });
        els.push(Element::text_color("Jumpers:", 40, 3, FONT_HEADER));

        let store = self.store.borrow();
        let np = store.profiles.num_profiles();
        let has_slot = store.profiles.has_slot();
        let entries = if has_slot { np + 1 } else { np };

        for i in 1..=np {
            let y = Self::y_for(i);
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

        // inc(temp,2), then "Back to Main Menu"
        let back_temp = if has_slot { np + 3 } else { np + 2 };
        els.push(Element::text_color("Back to Main Menu", 40, Self::y_for(back_temp), FONT_BACK));

        // highlight box: regular items use position formula, Back position uses text y - 3
        let (by, box_h) = if self.selected <= entries {
            (10 + ((self.selected - 1) * 8) as i32, 9)
        } else {
            (Self::y_for(back_temp) - 3, 9)
        };
        els.push(Element::box_(34, by, 123, box_h, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.editing {
            return match event {
                Event::Keyboard(Key::Enter) => {
                    let idx = self.selected;
                    if let Some(p) = self.store.borrow_mut().profiles.profiles.get_mut(idx) {
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

        let (_np, entries, has_slot) = {
            let store = self.store.borrow();
            let np = store.profiles.num_profiles();
            let has_slot = store.profiles.has_slot();
            let entries = if has_slot { np + 1 } else { np };
            (np, entries, has_slot)
        };
        let total = entries + 1; // Back is the exit slot

        match event {
            Event::Keyboard(Key::Up) if self.selected > 1 => {
                self.selected -= 1;
                None
            }
            Event::Keyboard(Key::Up) => {
                self.selected = total;
                None
            }
            Event::Keyboard(Key::Down) if self.selected < total => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selected = 1;
                None
            }
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                if self.selected <= entries {
                    if self.selected == entries && has_slot {
                        let mut store = self.store.borrow_mut();
                        let new_idx = store.profiles.num_profiles() + 1;
                        let mut p = Profile::default();
                        p.name = "SKI JUMPER".to_string();
                        let mut counter: usize = 2;
                        loop {
                            let conflict = store.profiles.profiles.iter().skip(1)
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
