use crate::competition::factory;
use crate::competition::koth::builder;
use crate::gfx::theme::{
    BG_PURPLE, BG_RED, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::layout::shorten_name;
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent};

use crate::text::lang::LangBase;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KothMode {
    Main,
    Packs,
    Opponents,
}

pub struct KothSetupView {
    resources: ResourcesRef,
    save_manager: SaveRef,
    selected: usize,
    mode: KothMode,
    pack_cursor: usize,
    selected_opponents: Vec<usize>,
    preview_opponent: usize,
}

impl KothSetupView {
    pub fn new(resources: ResourcesRef, save_manager: SaveRef) -> Self {
        Self {
            resources,
            save_manager,
            selected: 1,
            mode: KothMode::Main,
            pack_cursor: 0,
            selected_opponents: Vec::new(),
            preview_opponent: 0,
        }
    }

    fn update_config(&self, config: &mut Config, f: impl FnOnce(&mut Config)) {
        f(config);
        self.save_manager.save_config(config);
    }

    fn col1(&self, state: &GameState) -> engine::color::Rgba {
        if state.config.koth_pack > 0 {
            FONT_GRAY
        } else {
            FONT_BODY
        }
    }

    fn col2(&self, state: &GameState) -> engine::color::Rgba {
        if state.config.koth_pack > 0 {
            FONT_GRAY
        } else {
            FONT_GOLD
        }
    }

    fn paint_content(&self, paint: &mut PaintCx<'_>, state: &GameState) {
        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 169, 99), FILL_GRAY);
        paint.pattern_fill((0, 100, 169, 100), BG_RED);
        paint.pattern_fill((170, 0, 150, 200), BG_PURPLE);
        let lang = &self.resources.langbase;
        let cfg = &state.config;

        if self.mode == KothMode::Opponents {
            paint.pattern_fill((170, 0, 150, 200), BG_PURPLE);
            paint.text((180, 2), FONT_BODY, lang.tr(138));
            paint.text((180, 9), FONT_GRAY, lang.tr(139));
            paint.text((180, 16), FONT_GRAY, lang.tr(140));
            paint.text((180, 24), FONT_TEAL, lang.tr(141));
            paint.right_text((310, 24), FONT_TEAL, lang.tr(142));
        } else {
            // --- right panel: "Computer Jumpers:" (white, Pascal 240) ---
            paint.text((180, 10), FONT_BODY, lang.tr(120));
            if cfg.koth_opponent_count > 0 {
                for i in 0..cfg.koth_opponent_count.min(20) as usize {
                    let idx = cfg.koth_opponent_ids.get(i).copied().unwrap_or(0) as usize;
                    let name = self
                        .resources
                        .player_names(cfg.name_set_index as usize)
                        .get(idx - 1)
                        .map(|s| shorten_name(s, &self.resources.font, 110))
                        .unwrap_or_else(|| "?".to_string());
                    let y = (20 + (i + 1) * 8) as i32;
                    paint.text((180, y), FONT_GOLD, format!("{} #{}", name, idx));
                }
            } else {
                paint.text((180, 30), FONT_GOLD, lang.tr(9));
            }
        }

        // --- left panel: menu background (Pascal MakeMenu bgcolor=245) ---
        paint.pattern_fill((4, 7, 160, 63), FILL_GRAY);

        // --- left panel: menu items ---
        paint.text((10, 10), FONT_BODY, format!("1 - {}", lang.tr(121)));
        paint.text((10, 20), FONT_BODY, format!("2 - {}", lang.tr(122)));
        paint.text((10, 30), FONT_BODY, format!("3 - {}", lang.tr(123)));
        paint.text(
            (10, 40),
            self.col1(state),
            format!("4 - {}", lang.tr(124)),
        );
        let hill_name = if cfg.koth_hill < 0 {
            lang.tr(155)
        } else {
            self.resources
                .hills
                .hill(cfg.koth_hill as usize)
                .map(|h| h.name.as_str())
                .unwrap_or("?")
        };
        paint.text((80, 40), self.col2(state), hill_name);
        paint.text(
            (10, 50),
            self.col1(state),
            format!("5 - {}", lang.tr(125)),
        );
        let wind_str = if cfg.koth_wind != 0 {
            lang.tr(6)
        } else {
            lang.tr(7)
        };
        paint.text((80, 50), self.col2(state), wind_str);
        paint.text(
            (10, 60),
            self.col1(state),
            format!("6 - {}", lang.tr(126)),
        );
        paint.text(
            (80, 60),
            self.col2(state),
            lang.tr(cfg.koth_rounds as usize),
        );
        paint.text((10, 80), FONT_BODY, format!("0 - {}", lang.tr(127)));

        // --- left panel bottom: K.O.T.H Challenge Level (gold, Pascal 246) ---
        paint.text((10, 110), FONT_GOLD, lang.tr(130));

        // --- left panel bottom: pack list (Pascal kothchallenge) ---
        let is_pack_mode = self.mode == KothMode::Packs;
        let mut py = 120i32;
        for pack in 0..7u8 {
            let title = koth_pack_title(pack, lang);
            // In pack mode all items are white (Pascal kothchallenge(x,255))
            let color = if is_pack_mode {
                FONT_BODY
            } else {
                let selected_pack = if cfg.koth_pack == 0 {
                    6u8
                } else {
                    (cfg.koth_pack - 1) as u8
                };
                if selected_pack == pack {
                    FONT_BODY
                } else {
                    FONT_GRAY
                }
            };
            paint.text((10, py), color, title);
            py += 8;
            if pack == 5 {
                py += 8;
            }
        }

        match self.mode {
            KothMode::Main => {
                let sel = self.selected;
                if sel == 0 {
                    paint.stroke((4, 77, 160, 10), FONT_BODY);
                } else if sel <= 6 {
                    let sy = (10 + (sel - 1) * 10) as i32;
                    paint.stroke((4, sy - 3, 160, 10), FONT_BODY);
                }
            }
            KothMode::Packs => {
                let cur = self.pack_cursor;
                let pcy = pack_cursor_y(cur);
                paint.stroke((4, pcy - 3, 160, 10), FONT_BODY);
            }
            _ => {}
        }

        // Draw opponent rows — stack + preview (like CustomCupSetupView)
        if self.mode == KothMode::Opponents {
            let names = self.resources.player_names(cfg.name_set_index as usize);
            let sel = &self.selected_opponents;
            let prev = self.preview_opponent;
            // selected opponents in gold
            for (i, &id) in sel.iter().enumerate() {
                let name = names.get(id - 1).map(|s| s.as_str()).unwrap_or("?");
                let y = (i as i32 + 1) * 8 + 25;
                paint.fill((178, y - 2, 137, 10), BG_PURPLE);
                paint.text(
                    (180, y),
                    FONT_GOLD,
                    shorten_name(name, &self.resources.font, 110),
                );
                paint.right_text((310, y), FONT_GOLD, format!("#{}", id));
            }
            // preview slot at bottom (white)
            if sel.len() < 20 {
                let y = (sel.len() as i32 + 1) * 8 + 25;
                let name = names.get(prev).map(|s| s.as_str()).unwrap_or("?");
                paint.fill((178, y - 2, 137, 10), BG_PURPLE);
                paint.text(
                    (180, y),
                    FONT_BODY,
                    shorten_name(name, &self.resources.font, 110),
                );
                paint.right_text((310, y), FONT_BODY, format!("#{}", prev + 1));
            }
        }
    }

    fn handle_input(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        match self.mode {
            KothMode::Main => self.handle_main(event, state),
            KothMode::Packs => self.handle_packs(event, state),
            KothMode::Opponents => self.handle_opponents(event, state),
        }
    }
}

impl GameScreen for KothSetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event, cx.state) {
            nav.navigate(route);
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(paint, cx.state);
    }
}

impl KothSetupView {
    fn handle_main(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        match &event {
            UiEvent::KeyDown(Key::Escape) => Some(RouteTarget::MainMenu),
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.selected = if self.selected == 0 {
                    6
                } else {
                    self.selected - 1
                };
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.selected = if self.selected >= 6 {
                    0
                } else {
                    self.selected + 1
                };
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                self.activate(self.selected, state)
            }
            UiEvent::Text(ch) if *ch >= '0' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.selected = n;
                None
            }
            _ => None,
        }
    }

    fn handle_packs(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        match &event {
            UiEvent::KeyDown(Key::Escape) => {
                self.mode = KothMode::Main;
                None
            }
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.pack_cursor = if self.pack_cursor == 0 {
                    6
                } else {
                    self.pack_cursor - 1
                };
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.pack_cursor = if self.pack_cursor >= 6 {
                    0
                } else {
                    self.pack_cursor + 1
                };
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                let cur = self.pack_cursor;
                let pack = if cur == 0 { 0 } else { cur as i32 };
                self.apply_pack(pack, state);
                None
            }
            UiEvent::Text(ch) if *ch >= '1' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.pack_cursor = n;
                None
            }
            UiEvent::Text('0') => {
                self.pack_cursor = 0;
                None
            }
            _ => None,
        }
    }

    fn handle_opponents(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        let max_idx = self
            .resources
            .player_names(state.config.name_set_index as usize)
            .len();
        match event {
            UiEvent::KeyDown(Key::Escape) => {
                self.update_config(&mut state.config, |cfg| {
                    cfg.koth_opponent_ids =
                        self.selected_opponents.iter().map(|&v| v as i32).collect();
                    cfg.koth_opponent_count = self.selected_opponents.len() as i32;
                });
                self.mode = KothMode::Main;
                None
            }
            UiEvent::KeyDown(Key::Down) | UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                let prev_id = self.preview_opponent + 1;
                if self.selected_opponents.len() < 20 && !self.selected_opponents.contains(&prev_id)
                {
                    self.selected_opponents.push(prev_id);
                }
                // advance preview to first unselected after prev_id
                let mut next = prev_id % max_idx;
                for _ in 0..max_idx {
                    if !self.selected_opponents.contains(&(next + 1)) {
                        self.preview_opponent = next;
                        break;
                    }
                    next = (next + 1) % max_idx;
                }
                None
            }
            UiEvent::KeyDown(Key::Up) | UiEvent::KeyDown(Key::Backspace) => {
                if let Some(id) = self.selected_opponents.pop() {
                    self.preview_opponent = id - 1;
                }
                None
            }
            UiEvent::KeyDown(Key::Left) => {
                let prev = self.preview_opponent;
                let mut next = (prev + max_idx - 1) % max_idx;
                for _ in 0..max_idx {
                    if !self.selected_opponents.contains(&(next + 1)) {
                        self.preview_opponent = next;
                        break;
                    }
                    next = (next + max_idx - 1) % max_idx;
                }
                None
            }
            UiEvent::KeyDown(Key::Right) => {
                let prev = self.preview_opponent;
                let mut next = (prev + 1) % max_idx;
                for _ in 0..max_idx {
                    if !self.selected_opponents.contains(&(next + 1)) {
                        self.preview_opponent = next;
                        break;
                    }
                    next = (next + 1) % max_idx;
                }
                None
            }
            UiEvent::KeyDown(Key::Home) => {
                self.preview_opponent = 0;
                None
            }
            UiEvent::KeyDown(Key::End) => {
                self.preview_opponent = max_idx - 1;
                None
            }
            UiEvent::KeyDown(Key::PageUp) => {
                self.preview_opponent = self.preview_opponent.saturating_sub(10);
                None
            }
            UiEvent::KeyDown(Key::PageDown) => {
                self.preview_opponent = (self.preview_opponent + 10).min(max_idx - 1);
                None
            }
            _ => None,
        }
    }

    fn apply_pack(&mut self, pack: i32, state: &mut GameState) {
        self.update_config(&mut state.config, |cfg| cfg.koth_pack = pack);
        builder::apply_koth_pack(state, &self.save_manager, pack as u8);
        self.mode = KothMode::Main;
    }

    fn activate(&mut self, n: usize, state: &mut GameState) -> Option<RouteTarget> {
        match n {
            0 => Some(RouteTarget::MainMenu),
            1 => self.start_koth(state),
            2 => {
                let pack = state.config.koth_pack;
                self.pack_cursor = if pack == 0 { 0 } else { pack as usize };
                self.mode = KothMode::Packs;
                None
            }
            3 => {
                self.update_config(&mut state.config, |cfg| cfg.koth_pack = 0);
                self.selected_opponents = if state.config.koth_opponent_count > 0 {
                    state
                        .config
                        .koth_opponent_ids
                        .iter()
                        .map(|&v| v as usize)
                        .collect()
                } else {
                    Vec::new()
                };
                self.preview_opponent = 0;
                self.mode = KothMode::Opponents;
                None
            }
            4 => Some(RouteTarget::KothHillPicker),
            5 => {
                self.update_config(&mut state.config, |cfg| {
                    cfg.koth_wind = if cfg.koth_wind != 0 { 0 } else { 1 }
                });
                None
            }
            6 => {
                self.update_config(&mut state.config, |cfg| {
                    cfg.koth_rounds = if cfg.koth_rounds == 1 { 2 } else { 1 };
                });
                None
            }
            _ => None,
        }
    }

    fn start_koth(&self, state: &mut GameState) -> Option<RouteTarget> {
        let profiles = state.profiles.clone();
        let hill_count = self.resources.hills.len();
        let name_set_index = state.config.name_set_index as usize;
        let rng_clone = state.rng.clone();
        let comp = factory::koth(
            &state.config,
            &profiles,
            self.resources.player_names(name_set_index),
            hill_count,
            state.config.unique_computer_names != 0,
            rng_clone,
        );
        state.start_active(comp);
        state.wind.set_enabled(state.config.koth_wind != 0);
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
        0 => format!("1. {}", lang.tr(131)),
        1 => format!("2. {}", lang.tr(132)),
        2 => format!("3. {}", lang.tr(133)),
        3 => format!("4. {}", lang.tr(134)),
        4 => format!("5. {}", lang.tr(135)),
        5 => format!("6. {}", lang.tr(136)),
        6 => format!("0. {}", lang.tr(137)),
        _ => format!("{pack}. ?"),
    }
}
