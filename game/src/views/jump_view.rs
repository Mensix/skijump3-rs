use crate::data::hill_profile::HillTerrain;
use crate::jump::animation::{inrun_body_anim, slope_ski_anim, takeoff_body_anim};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpPhase {
    OnBar,
    Inrun,
}

#[derive(Debug, Clone)]
struct JumpState {
    phase: JumpPhase,
    matka: f64,
    px: f64,
    pxk: f64,
    maxspeed: f64,
    qx: f64,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    frame: i32,
    takeoff_counter: u8,
    takeoff_phase: u8,
}

impl JumpState {
    fn new(terrain: &HillTerrain, maxspeed: f64) -> Self {
        let matka = -f64::from(terrain.keula_x) + 10.0;
        let qx = f64::from(terrain.keula_x) + 0.5;
        let x = (matka + qx).round() as i32;
        let y = terrain.profiili(x);
        Self {
            phase: JumpPhase::OnBar,
            matka,
            px: 0.0,
            pxk: 1.016,
            maxspeed,
            qx,
            x,
            y,
            sx: 0,
            sy: 0,
            frame: 0,
            takeoff_counter: 0,
            takeoff_phase: 0,
        }
    }

    fn start(&mut self) {
        if self.phase == JumpPhase::OnBar {
            self.phase = JumpPhase::Inrun;
            self.px = 37.0;
            self.frame = 0;
        }
    }

    fn start_takeoff(&mut self) {
        if self.phase == JumpPhase::Inrun && self.matka > -40.0 && self.takeoff_counter == 0 {
            self.takeoff_counter = 1;
        }
    }

    fn tick(&mut self, terrain: &HillTerrain) {
        self.frame += 1;
        if self.phase != JumpPhase::Inrun {
            return;
        }

        let fx = self.x;
        let fy = self.y;
        self.matka += self.px * 0.01;
        self.x = (self.matka + self.qx).round() as i32;
        self.y = terrain.profiili(self.x);
        self.px = (self.px * self.pxk).min(self.maxspeed);

        if self.takeoff_counter > 0 {
            self.takeoff_counter = self.takeoff_counter.saturating_add(1);
            self.px += 0.21;
        }

        if self.x >= 160 && self.x < 864 {
            self.sx += self.x - fx;
        }
        if self.y >= 100 && self.y < 412 {
            self.sy += self.y - fy;
        }
        self.sx = self.sx.min(704);
        self.sy = self.sy.min(312);
    }

    fn anims(&mut self, terrain: &HillTerrain) -> (u16, u16) {
        match self.phase {
            JumpPhase::OnBar => (163, slope_ski_anim(terrain.maki_kulma(self.x))),
            JumpPhase::Inrun => {
                let ski = slope_ski_anim(terrain.maki_kulma(self.x));
                if self.takeoff_counter > 0 {
                    (takeoff_body_anim(&mut self.takeoff_phase), ski)
                } else {
                    (inrun_body_anim(ski), ski)
                }
            }
        }
    }

    fn status(&self) -> &'static str {
        match self.phase {
            JumpPhase::OnBar => "ON BAR - ENTER/RIGHT TO START",
            JumpPhase::Inrun if self.takeoff_counter > 0 => "TAKEOFF",
            JumpPhase::Inrun => "INRUN - UP TO TAKE OFF",
        }
    }
}

pub struct JumpView {
    resources: ResourcesRef,
    hill_idx: usize,
    terrain: Result<HillTerrain, String>,
    state: RefCell<Option<JumpState>>,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = *store.selected_hill.borrow();
        let hill = resources.hills.hill(hill_idx);
        let terrain = hill
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);
        let state = match (&terrain, hill) {
            (Ok(terrain), Some(hill)) => Some(JumpState::new(terrain, hill.vx_final as f64)),
            _ => None,
        };

        Self {
            resources,
            hill_idx,
            terrain,
            state: RefCell::new(state),
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

        let mut state_ref = self.state.borrow_mut();
        let state = state_ref.as_mut().expect("terrain-loaded jump state");
        state.tick(terrain);

        let viewport = terrain.viewport_pixels(state.sx, state.sy, WIDTH, HEIGHT);
        let mut els = vec![Element::image_region(ImageRegion {
            pixels: Rc::clone(&viewport),
            src_w: WIDTH,
            src_h: HEIGHT,
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

        els.push(Element::text_color(state.status(), 8, 38, FONT_HELP));

        let jumper_x = state.x - state.sx;
        let jumper_y = state.y - state.sy;
        let (body_anim, ski_anim) = state.anims(terrain);
        els.push(Element::sprite(body_anim, jumper_x, jumper_y - 2));
        els.push(Element::sprite(ski_anim, jumper_x, jumper_y - 1));
        els.push(Element::text_color("ESC: HILLS", 8, 188, FONT_HELP));
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Practice),
            Event::Keyboard(Key::Enter | Key::Right) => {
                if let Some(state) = self.state.get_mut() {
                    state.start();
                }
                None
            }
            Event::Keyboard(Key::Up) => {
                if let Some(state) = self.state.get_mut() {
                    state.start_takeoff();
                }
                None
            }
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        if let Ok(terrain) = &self.terrain {
            terrain.apply_hill_palette(palette);
        }
    }
}
