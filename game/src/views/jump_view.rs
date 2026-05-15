use crate::data::hill_profile::HillTerrain;
use crate::jump::presentation;
use crate::jump::{JumpPolicy, JumpPresentationContext, JumpSession, WindGaugePosition};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::training_jump_controller::{TrainingJumpAction, TrainingJumpController};
use engine::consts::{HEIGHT, WIDTH};
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::{Cell, RefCell};
use std::path::Path;

#[derive(Debug, Clone)]
enum SaveField {
    Author,
    Name,
    Filename,
}

#[derive(Debug, Clone)]
enum SaveDialogState {
    Inactive,
    Browse {
        selected: usize,
    },
    EditField {
        field: SaveField,
        value: String,
        cursor: usize,
    },
    ConfirmOverwrite {
        filename: String,
        _author: String,
        _name: String,
    },
}

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    hill_idx: usize,
    session: RefCell<JumpSession>,
    jumper_name: String,
    save_dialog: RefCell<SaveDialogState>,
    save_author: RefCell<String>,
    save_name: RefCell<String>,
    save_filename: RefCell<String>,
    cursor_blink: Cell<u32>,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = *store.selected_hill.borrow();
        let hill = resources.hills.hill(hill_idx);
        let terrain = hill
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);

        let mut snow = SnowSystem::new();

        // Pascal: each new practice/competition round resets eka=true
        *store.eka.borrow_mut() = true;

        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, *store.wind_place.borrow());

            if *store.eka.borrow() {
                // Pascal lines 1127-1131: snow LMaara calc + VieLmaara on first jump
                let lmaara = rng.random_i32(2) * rng.random_i32(256);
                let lmaara = if lmaara > 0 && lmaara < 40 {
                    lmaara + rng.random_i32(150)
                } else {
                    lmaara
                };
                let lmaara = if lmaara > 0 && rng.random_i32(4) == 0 {
                    lmaara + 1000
                } else {
                    lmaara
                };
                snow.set_count(lmaara as u16, &mut rng);

                // Pascal line 1149: Tuuli.Hae inside eka block (first wind shift)
                wind.sample(&mut rng);

                *store.eka.borrow_mut() = false;
            } else {
                // Pascal: on subsequent jumps, snow persists (no re-init).
                // Initialize with fixed count so snow stays visible.
                snow.set_count(50, &mut rng);
            }
        }

        let jumper_name = "TRAINEE".to_string();
        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map(|r| r.len as i32)
            .unwrap_or(0);
        let session = JumpSession::new(
            terrain,
            hill,
            hill_idx,
            *store.start_gate.borrow(),
            snow,
            jumper_name.clone(),
            JumpPolicy::training(),
            record_distance,
        );

        Self {
            resources,
            store,
            hill_idx,
            session: RefCell::new(session),
            jumper_name,
            save_dialog: RefCell::new(SaveDialogState::Inactive),
            save_author: RefCell::new(String::new()),
            save_name: RefCell::new(String::new()),
            save_filename: RefCell::new("TEMP".to_string()),
            cursor_blink: Cell::new(0),
        }
    }

    fn enter_save_dialog(&self) {
        let hill_name = self
            .resources
            .hills
            .hill(self.hill_idx)
            .map(|h| h.name.clone())
            .unwrap_or_default();
        let pb = self.store.profiles.borrow();
        let author_name = pb
            .active_order
            .first()
            .and_then(|&idx| {
                let p = pb.profiles.get(idx)?;
                let name = if p.real_name.is_empty() {
                    p.name.clone()
                } else {
                    p.real_name.clone()
                };
                Some(name)
            })
            .unwrap_or_default();
        *self.save_author.borrow_mut() = author_name;
        *self.save_name.borrow_mut() = format!("Huge Jump in {}", hill_name);
        *self.save_filename.borrow_mut() = "TEMP".to_string();
        *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: 0 };
    }

    fn do_save_replay(&self) {
        let session = self.session.borrow();
        if let Some(trace) = session.replay_trace() {
            let filename = format!("{}.SJR", self.save_filename.borrow());
            let _ = std::fs::write(&filename, trace.to_sjr_bytes());
        }
        *self.save_dialog.borrow_mut() = SaveDialogState::Inactive;
    }

    fn reset_jump_state(&self) {
        if let Some(hill) = self.resources.hills.hill(self.hill_idx) {
            // Pascal: wind continues between jumps, NOT re-initialized (only F5 resets it)
            let record_distance = self
                .store
                .records
                .borrow()
                .hill_record(self.hill_idx)
                .map(|r| r.len as i32)
                .unwrap_or(0);
            self.session.borrow_mut().reset_state(
                hill,
                *self.store.start_gate.borrow(),
                record_distance,
            );
        }
    }

    fn reset_wind(&self) {
        {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            wind.initialize(&mut rng, *self.store.wind_place.borrow());
        }
    }

    fn handle_jump_event(&mut self, event: Event) -> Option<RouteTarget> {
        match TrainingJumpController.handle_event(event, self.session.get_mut()) {
            TrainingJumpAction::None => None,
            TrainingJumpAction::RoutePractice => Some(RouteTarget::Practice),
            TrainingJumpAction::ResetWind => {
                self.reset_wind();
                None
            }
            TrainingJumpAction::ResetJump => {
                let _ = self.session.get_mut().outcome();
                let _ = self.session.get_mut().replay_trace();
                self.reset_jump_state();
                None
            }
            TrainingJumpAction::PersistStartGate(start_gate) => {
                *self.store.start_gate.borrow_mut() = start_gate;
                None
            }
            TrainingJumpAction::SaveReplay => {
                self.enter_save_dialog();
                None
            }
        }
    }

    fn handle_save_dialog_event(&self, event: Event) -> Option<RouteTarget> {
        let state = self.save_dialog.borrow().clone();
        match state {
            SaveDialogState::Browse { selected } => match event {
                Event::Keyboard(Key::Escape) => {
                    *self.save_dialog.borrow_mut() = SaveDialogState::Inactive;
                    None
                }
                Event::Keyboard(Key::Up) if selected > 0 => {
                    let next = if selected == 4 { 3 } else { selected - 1 };
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: next };
                    None
                }
                Event::Keyboard(Key::Up) => {
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: 4 };
                    None
                }
                Event::Keyboard(Key::Down) if selected < 4 => {
                    let next = if selected == 3 { 4 } else { selected + 1 };
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: next };
                    None
                }
                Event::Keyboard(Key::Down) => {
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: 0 };
                    None
                }
                Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => match selected {
                    0 => {
                        let v = self.save_author.borrow().clone();
                        let len = v.len();
                        self.cursor_blink.set(0);
                        *self.save_dialog.borrow_mut() = SaveDialogState::EditField {
                            field: SaveField::Author,
                            value: v,
                            cursor: len,
                        };
                        None
                    }
                    1 => {
                        let v = self.save_name.borrow().clone();
                        let len = v.len();
                        self.cursor_blink.set(0);
                        *self.save_dialog.borrow_mut() = SaveDialogState::EditField {
                            field: SaveField::Name,
                            value: v,
                            cursor: len,
                        };
                        None
                    }
                    2 => {
                        let v = self.save_filename.borrow().clone();
                        let len = v.len();
                        self.cursor_blink.set(0);
                        *self.save_dialog.borrow_mut() = SaveDialogState::EditField {
                            field: SaveField::Filename,
                            value: v,
                            cursor: len,
                        };
                        None
                    }
                    3 => {
                        // Don't save / cancel
                        *self.save_dialog.borrow_mut() = SaveDialogState::Inactive;
                        None
                    }
                    4 => {
                        // Save
                        let filename = self.save_filename.borrow().clone();
                        if Path::new(&format!("{}.SJR", filename)).exists() {
                            let author = self.save_author.borrow().clone();
                            let name = self.save_name.borrow().clone();
                            *self.save_dialog.borrow_mut() = SaveDialogState::ConfirmOverwrite {
                                filename,
                                _author: author,
                                _name: name,
                            };
                        } else {
                            self.do_save_replay();
                        }
                        None
                    }
                    _ => None,
                },
                _ => None,
            },
            SaveDialogState::EditField {
                field,
                mut value,
                cursor,
            } => match event {
                Event::Keyboard(Key::Escape) => {
                    self.cursor_blink.set(0);
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse {
                        selected: field_idx(&field),
                    };
                    None
                }
                Event::Keyboard(Key::Enter) => {
                    self.cursor_blink.set(0);
                    match field {
                        SaveField::Author => *self.save_author.borrow_mut() = value,
                        SaveField::Name => *self.save_name.borrow_mut() = value,
                        SaveField::Filename => *self.save_filename.borrow_mut() = value,
                    }
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse {
                        selected: field_idx(&field),
                    };
                    None
                }
                Event::Keyboard(Key::Backspace) if cursor > 0 => {
                    self.cursor_blink.set(0);
                    value.remove(cursor - 1);
                    *self.save_dialog.borrow_mut() = SaveDialogState::EditField {
                        field,
                        value,
                        cursor: cursor - 1,
                    };
                    None
                }
                Event::Keyboard(Key::Char(c)) => {
                    let limit = match field {
                        SaveField::Filename => 8,
                        _ => 130,
                    };
                    if value.len() >= limit {
                        return None;
                    }
                    self.cursor_blink.set(0);
                    value.insert(cursor, c);
                    *self.save_dialog.borrow_mut() = SaveDialogState::EditField {
                        field,
                        value,
                        cursor: cursor + 1,
                    };
                    None
                }
                _ => None,
            },
            SaveDialogState::ConfirmOverwrite { .. } => match event {
                Event::Keyboard(Key::Escape) => {
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: 3 };
                    None
                }
                Event::Keyboard(Key::Char('y') | Key::Char('Y')) => {
                    self.do_save_replay();
                    None
                }
                Event::Keyboard(Key::Char('n') | Key::Char('N')) => {
                    *self.save_dialog.borrow_mut() = SaveDialogState::Browse { selected: 2 };
                    None
                }
                _ => None,
            },
            SaveDialogState::Inactive => None,
        }
    }
}

fn field_idx(field: &SaveField) -> usize {
    match field {
        SaveField::Author => 0,
        SaveField::Name => 1,
        SaveField::Filename => 2,
    }
}

impl JumpView {
    fn save_dialog_elements(&self) -> Vec<Element> {
        // Pascal replayinfo: newscreen(1,0) + logo sprite
        let mut els = vec![
            Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0),
            Element::fillbox(0, 0, WIDTH as i32, 19, 245),
            Element::fillbox(0, 20, WIDTH as i32, (HEIGHT - 20) as i32, 243),
            Element::FillArea { thing: 63 },
            Element::sprite(61, 80, 6),
        ];

        match self.save_dialog.borrow().clone() {
            SaveDialogState::Browse { .. } | SaveDialogState::EditField { .. } => {
                let selected_idx = match *self.save_dialog.borrow() {
                    SaveDialogState::Browse { selected } => selected,
                    SaveDialogState::EditField { ref field, .. } => field_idx(field),
                    _ => 0,
                };
                let editing = matches!(
                    *self.save_dialog.borrow(),
                    SaveDialogState::EditField { .. }
                );
                let editing_field = match *self.save_dialog.borrow() {
                    SaveDialogState::EditField { ref field, .. } => Some(field_idx(field)),
                    _ => None,
                };

                let distance = self
                    .session
                    .borrow()
                    .outcome()
                    .map(|o| format!("{:.1}", o.distance as f64 / 10.0))
                    .unwrap_or_default();
                let hill_name = self
                    .resources
                    .hills
                    .hill(self.hill_idx)
                    .map(|h| format!("{} K{}", h.name, h.kr))
                    .unwrap_or_default();

                // Header: Pascal writefont(30,6,lstr(25)+': '+txtp(hp)+' at '+hillname+' K'+txt(hillkr))
                els.push(Element::text_color(
                    format!(
                        "{}: {}m at {}",
                        self.resources.langbase.lstr(25),
                        distance,
                        hill_name
                    ),
                    30,
                    6,
                    FONT_DEFAULT,
                ));

                // Pascal: for temp:=1 to 5 do, yy:=(temp*16)+26 → i*16+42 for 0-based i
                for i in 0..5 {
                    let yy = (i * 16 + 42) as i32;
                    let final_yy = if i == 4 { yy + 16 } else { yy };

                    // Label color: temp<5 → FONT_DEFAULT(240), temp=5 → FONT_GOLD(246, stays from prev)
                    let label_color = if i < 4 { FONT_DEFAULT } else { FONT_GOLD };

                    // Label string
                    let label = match i {
                        0..=2 => format!("{}. {}", i + 1, self.resources.langbase.lstr(291 + i)),
                        3 => format!("4. {}", self.resources.langbase.lstr(295)),
                        4 => format!("5. {}", self.resources.langbase.lstr(296)),
                        _ => String::new(),
                    };
                    els.push(Element::text_color(&label, 18, final_yy, label_color));

                    // Value in gold (FONT_GOLD=246) for items 1-3 only
                    if i < 3 {
                        let is_editing = editing && editing_field == Some(i);
                        let value = if is_editing {
                            match *self.save_dialog.borrow() {
                                SaveDialogState::EditField { ref value, .. } => value.clone(),
                                _ => String::new(),
                            }
                        } else {
                            match i {
                                0 => self.save_author.borrow().clone(),
                                1 => self.save_name.borrow().clone(),
                                2 => self.save_filename.borrow().clone(),
                                _ => String::new(),
                            }
                        };
                        if is_editing {
                            // Pascal getstr: fillbox(xx-2,yy-2,xx+maxlength+2,yy+7,bkcolor)
                            // Pascal getstr2 (filename): fillbox(xx-2,yy-2,xx+(maxchars*7)+2,yy+8,bkcolor)
                            let (fw, fh) = match editing_field {
                                Some(2) => (60, 11), // filename: 8*7+4=60px
                                _ => (134, 10),      // author/name: 130+4=134px
                            };
                            els.push(Element::fillbox(146, final_yy - 2, fw, fh, 242));
                        }
                        els.push(Element::text_color(&value, 148, final_yy, FONT_GOLD));

                        if is_editing {
                            if let SaveDialogState::EditField {
                                ref value, cursor, ..
                            } = *self.save_dialog.borrow()
                            {
                                // Cursor: Pascal givech 5x1 horizontal line at (xx+cx, yy+6), white, blinks 11on/10off
                                let cx = 148
                                    + self
                                        .resources
                                        .font
                                        .string_width(&value[..cursor.min(value.len())])
                                        as i32;
                                let blink = self.cursor_blink.get();
                                self.cursor_blink.set(blink + 1);
                                if blink % 21 <= 10 {
                                    els.push(Element::fillbox(
                                        cx,
                                        final_yy + 6,
                                        5,
                                        1,
                                        FONT_DEFAULT,
                                    ));
                                }
                            }
                        }
                    }
                }

                // Selection box matching Pascal MakeMenu(15,39,135,16,4,...)
                // Items 1-4 at yy=36,52,68,84. EXIT (index 6) at yy=116 (skip gap at index 5/yy=100)
                let box_y = if selected_idx < 4 {
                    36 + selected_idx * 16
                } else {
                    36 + 5 * 16
                };
                els.push(Element::box_(9, box_y as i32, 135, 17, FONT_DEFAULT));
            }
            SaveDialogState::ConfirmOverwrite { ref filename, .. } => {
                // Pascal alertbox background
                els.push(Element::fillbox(59, 79, 203, 53, 242));
                els.push(Element::fillbox(60, 80, 201, 51, 244));
                els.push(Element::FillArea { thing: 63 });

                els.push(Element::text_color(
                    format!("{}.SJR {}", filename, self.resources.langbase.lstr(345)),
                    80,
                    90,
                    FONT_DEFAULT,
                ));
                els.push(Element::text_color(
                    format!("{} (Y/N):", self.resources.langbase.lstr(346)),
                    80,
                    110,
                    FONT_DEFAULT,
                ));
            }
            SaveDialogState::Inactive => {}
        }

        els
    }
}

impl JumpView {
    #[allow(unused_mut)]
    fn build_elements(&self) -> Vec<Element> {
        let mut session = self.session.borrow_mut();
        let Err(err) = session.terrain() else {
            if session.state().is_some() {
                return self.elements_for_loaded_session(&mut session);
            }
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            els.push(Element::text_color(
                "jump state not available",
                20,
                80,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        };

        let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
        els.push(Element::text_color(err, 20, 80, FONT_DEFAULT));
        els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
        els
    }
}

impl JumpView {
    fn elements_for_loaded_session(&self, session: &mut JumpSession) -> Vec<Element> {
        if session.phase().is_none() {
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            els.push(Element::text_color(
                "jump state not available",
                20,
                80,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        }

        let mut rng = self.store.rng.borrow_mut();
        let mut wind_store = self.store.wind.borrow_mut();
        let wind = session.tick_with_wind(&mut rng, &mut wind_store);
        drop(wind_store);
        drop(rng);

        let hill_name_k = self
            .resources
            .hills
            .hill(self.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = self.store.wind.borrow().position();
        let records = self.store.records.borrow();
        let frame = session
            .render_frame(wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        let ctx = JumpPresentationContext {
            font: &self.resources.font,
            langbase: &self.resources.langbase,
            jumper_name: &self.jumper_name,
            hill_name_k: &hill_name_k,
            hill_record: records.hill_record(self.hill_idx),
            wind_position: WindGaugePosition {
                x: wind_pos.x,
                y: wind_pos.y,
            },
        };
        presentation::elements(&frame, &ctx)
    }
}

impl View<RouteTarget> for JumpView {
    fn elements(&self) -> Vec<Element> {
        match *self.save_dialog.borrow() {
            SaveDialogState::Inactive => self.build_elements(),
            _ => self.save_dialog_elements(),
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let dialog_active = !matches!(*self.save_dialog.borrow(), SaveDialogState::Inactive);
        if dialog_active {
            self.handle_save_dialog_event(event)
        } else {
            self.handle_jump_event(event)
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if !matches!(*self.save_dialog.borrow(), SaveDialogState::Inactive) {
            return;
        }
        if let Ok(mut session) = self.session.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            let draw = session.draws_snow();
            session.render_snow(framebuffer, wind, draw);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if let SaveDialogState::Inactive = *self.save_dialog.borrow() {
            if let Ok(terrain) = self.session.borrow().terrain() {
                terrain.apply_hill_palette(palette);
            }
            palette.set(253, [10, 54, 10]);
            palette.set(254, [0, 47, 0]);
        }
    }
}
