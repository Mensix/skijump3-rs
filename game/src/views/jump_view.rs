use crate::data::hill_profile::HillTerrain;
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::rc::Rc;

pub struct JumpView {
    resources: ResourcesRef,
    hill_idx: usize,
    terrain: Result<HillTerrain, String>,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = *store.selected_hill.borrow();
        let terrain = resources
            .hills
            .hill(hill_idx)
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);

        Self {
            resources,
            hill_idx,
            terrain,
        }
    }

    fn hill_header(&self, els: &mut Vec<Element>) {
        if let Some(hill) = self.resources.hills.hill(self.hill_idx) {
            els.push(Element::text_color(
                format!("{} K{}", hill.name, hill.kr),
                8,
                8,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                format!(
                    "FRONT{}  VX{}  PK{:.2}  PL{:.4}",
                    hill.front_index,
                    hill.vx_final,
                    hill.pk(),
                    hill.pl_save()
                ),
                8,
                18,
                FONT_HELP,
            ));
        }
    }
}

impl View<RouteTarget> for JumpView {
    fn elements(&self) -> Vec<Element> {
        let Ok(terrain) = &self.terrain else {
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            if let Err(err) = &self.terrain {
                els.push(Element::text_color(err, 20, 80, FONT_DEFAULT));
            }
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        };

        let mut els = vec![Element::image_region(ImageRegion {
            pixels: Rc::clone(&terrain.pixels),
            src_w: terrain.width as u32,
            src_h: terrain.height as u32,
            src_x: 0,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            w: WIDTH,
            h: HEIGHT,
        })];

        self.hill_header(&mut els);
        els.push(Element::text_color(
            format!("KEULAX {}", terrain.keula_x),
            8,
            28,
            FONT_HELP,
        ));

        let jumper_x = 10;
        let jumper_y = terrain.profiili(jumper_x);
        els.push(Element::sprite(164, jumper_x, jumper_y - 2));
        els.push(Element::sprite(71, jumper_x, jumper_y - 1));
        els.push(Element::text_color("ESC: HILLS", 8, 188, FONT_HELP));
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape | Key::Enter) => Some(RouteTarget::Practice),
            _ => None,
        }
    }
}
