use engine::ui::{Element, Event, Key, View};
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
}

impl ProfilesView {
    pub fn new(store: StoreRef) -> Self {
        Self { store, selected: 1 }
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
            if let Some(p) = store.profiles.profiles.get(i) {
                els.push(Element::text_color(&p.name, 40, y, FONT_NAME));
            }
        }

        if has_slot {
            let y = self.y_for(np + 1);
            els.push(Element::text_color("*Create New Jumper*", 40, y, FONT_NEW));
        }

        // "Back to Main Menu" — inc(temp,2) after last entry
        let idx = if has_slot { np + 3 } else { np + 2 };
        els.push(Element::text_color("Back to Main Menu", 40, self.y_for(idx), FONT_BACK));

        // highlight box: MakeMenu(40,13,122,8) → xx=34, yy=10+(selected-1)*8
        let by = 10 + ((self.selected - 1) * 8) as i32;
        els.push(Element::box_(34, by, 123, 9, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let (np, entries) = {
            let store = self.store.borrow();
            let np = store.profiles.num_profiles();
            let has_slot = store.profiles.has_slot();
            let entries = if has_slot { np + 1 } else { np };
            (np, entries)
        };
        let total = entries + 1;

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
                    let idx = if self.selected <= np { self.selected } else { np + 1 };
                    self.store.borrow_mut().profiles.edit_index = idx;
                    Some(RouteTarget::ProfileEditor)
                } else {
                    Some(RouteTarget::MainMenu)
                }
            }
            Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
            _ => None,
        }
    }
}
