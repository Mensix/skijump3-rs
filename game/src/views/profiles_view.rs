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

    fn y_for(&self, temp: usize) -> i32 {
        (temp * 8 + 4) as i32
    }
}

impl View<RouteTarget> for ProfilesView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        // black canvas + two panels
        els.push(Element::fillbox(0, 0, 320, 200, 0));
        els.push(Element::fillbox(0, 0, 159, 200, BG_LEFT));
        els.push(Element::fillbox(160, 0, 160, 200, BG_RIGHT));
        els.push(Element::FillArea { thing: 63 });

        // "Jumpers:" at (40,3) fontcolor(241)
        els.push(Element::text_color("Jumpers:", 40, 3, FONT_HEADER));

        let store = self.store.borrow();
        let np = store.profiles.num_profiles();
        let has_slot = store.profiles.has_slot();

        // profile names with order number boxes
        for i in 1..=np {
            let y = self.y_for(i);
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

        // "*Create New Jumper*" if room
        if has_slot {
            let y = self.y_for(np + 1);
            els.push(Element::text_color("*Create New Jumper*", 40, y, FONT_NEW));
        }

        // "Back to Main Menu" — inc(temp,2) after last entry
        let idx = if has_slot { np + 3 } else { np + 2 };
        els.push(Element::text_color("Back to Main Menu", 40, self.y_for(idx), FONT_BACK));

        // highlight box
        let by = 10 + ((self.selected - 1) * 8) as i32;
        els.push(Element::box_(34, by, 123, 9, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let (np, entries, has_slot) = {
            let store = self.store.borrow();
            let np = store.profiles.num_profiles();
            let has_slot = store.profiles.has_slot();
            let entries = if has_slot { np + 1 } else { np };
            (np, entries, has_slot)
        };
        let total = entries + 1;

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
                    // if newly created with empty name, remove it
                    if self.edit_buf.is_empty() && self.selected > np.saturating_sub(1) {
                        // don't delete — just leave as unnamed
                    }
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
                        // Create new jumper
                        let mut store = self.store.borrow_mut();
                        let new_idx = store.profiles.num_profiles() + 1;
                        store.profiles.profiles.push(Profile::default());
                        store.profiles.edit_index = new_idx;
                        self.selected = new_idx;
                        self.edit_buf = "SKI JUMPER".to_string();
                        self.editing = true;
                        None
                    } else {
                        // Edit existing jumper
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
