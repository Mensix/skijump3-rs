use crate::competition::factory;
use crate::competition::koth::builder;
use crate::gfx::theme::{
    BG_PURPLE, BG_RED, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::screen::{GameCx, GameScreen, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::layout::shorten_name;
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenEventCx, UiEvent};
use engine::audio::Beep;

use crate::text::lang::LangBase;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KothMode {
    Main,
    Packs,
    Opponents,
}

pub struct KothSetupView {
    resources: ResourcesRef,
    selected: usize,
    mode: KothMode,
    pack_cursor: usize,
    selected_opponents: Vec<usize>,
    preview_opponent: usize,
}

impl KothSetupView {
    pub fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            selected: 1,
            mode: KothMode::Main,
            pack_cursor: 0,
            selected_opponents: Vec::new(),
            preview_opponent: 0,
        }
    }

    fn update_config(&self, config: &mut Config, f: impl FnOnce(&mut Config), cx: &Persistence) {
        f(config);
        cx.save_config(config);
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

    fn paint_content(&self, paint: &mut dyn UiCanvas, state: &GameState) {
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
            paint.text((180, 10), FONT_BODY, lang.tr(120));
            if cfg.koth_opponent_count > 0 {
                let names = self.resources.player_names(cfg.name_set_index as usize);
                let configured = cfg
                    .koth_opponent_ids
                    .iter()
                    .take(cfg.koth_opponent_count.max(0) as usize)
                    .filter_map(|&id| usize::try_from(id).ok())
                    .filter(|&idx| idx < names.len())
                    .collect::<Vec<_>>();
                for (i, idx) in unique_opponents(&configured).iter().copied().enumerate() {
                    let name = self
                        .resources
                        .player_names(cfg.name_set_index as usize)
                        .get(idx)
                        .map(|s| shorten_name(s, &self.resources.font, 110))
                        .unwrap_or_else(|| "?".to_string());
                    let y = (20 + (i + 1) * 8) as i32;
                    paint.text((180, y), FONT_GOLD, &format!("{} #{}", name, idx + 1));
                }
            } else {
                paint.text((180, 30), FONT_GOLD, lang.tr(9));
            }
        }

        paint.pattern_fill((4, 7, 160, 63), FILL_GRAY);

        paint.text((10, 10), FONT_BODY, &format!("1 - {}", lang.tr(121)));
        paint.text((10, 20), FONT_BODY, &format!("2 - {}", lang.tr(122)));
        paint.text((10, 30), FONT_BODY, &format!("3 - {}", lang.tr(123)));
        paint.text((10, 40), self.col1(state), &format!("4 - {}", lang.tr(124)));
        let hill_name = if cfg.koth_hill < 0 {
            lang.tr(155).to_string()
        } else {
            self.resources
                .hills
                .hill(cfg.koth_hill as usize)
                .map(|h| h.name.clone())
                .unwrap_or_else(|| "?".to_string())
        };
        paint.text((80, 40), self.col2(state), &hill_name);
        paint.text((10, 50), self.col1(state), &format!("5 - {}", lang.tr(125)));
        let wind_str = if cfg.koth_wind != 0 {
            lang.tr(6)
        } else {
            lang.tr(7)
        };
        paint.text((80, 50), self.col2(state), wind_str);
        paint.text((10, 60), self.col1(state), &format!("6 - {}", lang.tr(126)));
        paint.text(
            (80, 60),
            self.col2(state),
            lang.tr(cfg.koth_rounds as usize),
        );
        paint.text((10, 80), FONT_BODY, &format!("0 - {}", lang.tr(127)));

        paint.text((10, 110), FONT_GOLD, lang.tr(130));

        let is_pack_mode = self.mode == KothMode::Packs;
        let mut py = 120i32;
        for row in 0..7 {
            let pack = pack_for_row(row);
            let title = koth_pack_title(pack, lang);

            let color = if is_pack_mode {
                if self.pack_cursor == row {
                    FONT_GOLD
                } else {
                    FONT_BODY
                }
            } else if cfg.koth_pack == i32::from(pack) {
                FONT_GOLD
            } else {
                FONT_BODY
            };
            paint.text((10, py), color, &title);
            py += 8;
            if row == 5 {
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

        if self.mode == KothMode::Opponents {
            let names = self.resources.player_names(cfg.name_set_index as usize);
            let sel = &self.selected_opponents;
            let prev = self.preview_opponent;

            for (i, &id) in sel.iter().enumerate() {
                let name = names.get(id).map(|s| s.as_str()).unwrap_or("?");
                let y = (i as i32 + 1) * 8 + 25;
                paint.fill((178, y - 2, 137, 10), BG_PURPLE);
                paint.text(
                    (180, y),
                    FONT_GOLD,
                    &shorten_name(name, &self.resources.font, 110),
                );
                paint.right_text((310, y), FONT_GOLD, &format!("#{}", id + 1));
            }

            if sel.len() < 20 {
                let y = (sel.len() as i32 + 1) * 8 + 25;
                let name = names.get(prev).map(|s| s.as_str()).unwrap_or("?");
                paint.fill((178, y - 2, 137, 10), BG_PURPLE);
                paint.text(
                    (180, y),
                    FONT_BODY,
                    &shorten_name(name, &self.resources.font, 110),
                );
                paint.right_text((310, y), FONT_BODY, &format!("#{}", prev + 1));
            }
        }
    }

    fn handle_input(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> Option<RouteTarget> {
        match self.mode {
            KothMode::Main => self.handle_main(event, state, cx),
            KothMode::Packs => self.handle_packs(event, state, cx),
            KothMode::Opponents => self.handle_opponents(event, state, cx),
        }
    }
}

impl GameScreen for KothSetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::KeyDown(Key::F10)) {
            nav.back();
            return;
        }
        let persistence = cx.persistence();
        if let Some(route) = self.handle_input(event, cx.state, &persistence) {
            nav.navigate(route);
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint, cx.state);
    }
}

impl KothSetupView {
    const CURSOR_ITEMS: usize = 7;

    fn cycle_cursor(value: usize, delta: i32) -> usize {
        (value as i32 + delta).rem_euclid(Self::CURSOR_ITEMS as i32) as usize
    }

    fn handle_main(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> Option<RouteTarget> {
        match &event {
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.selected = Self::cycle_cursor(self.selected, -1);
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.selected = Self::cycle_cursor(self.selected, 1);
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                self.activate(self.selected, state, cx)
            }
            UiEvent::Text(ch) if *ch >= '0' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.selected = n;
                None
            }
            _ => None,
        }
    }

    fn step_preview_opponent(&mut self, max_idx: usize, delta: i32) {
        let mut next = (self.preview_opponent as i32 + delta).rem_euclid(max_idx as i32) as usize;
        for _ in 0..max_idx {
            if !self.selected_opponents.contains(&next) {
                self.preview_opponent = next;
                break;
            }
            next = (next as i32 + delta).rem_euclid(max_idx as i32) as usize;
        }
    }

    fn handle_packs(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> Option<RouteTarget> {
        match &event {
            UiEvent::KeyDown(Key::Escape) => {
                self.mode = KothMode::Main;
                None
            }
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.pack_cursor = Self::cycle_cursor(self.pack_cursor, -1);
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.pack_cursor = Self::cycle_cursor(self.pack_cursor, 1);
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                let cur = self.pack_cursor;
                self.apply_pack(i32::from(pack_for_row(cur)), state, cx);
                None
            }
            UiEvent::Text(ch) if *ch >= '1' && *ch <= '6' => {
                let n = *ch as usize - '0' as usize;
                self.pack_cursor = n - 1;
                None
            }
            UiEvent::Text('0') => {
                self.pack_cursor = 6;
                None
            }
            _ => None,
        }
    }

    fn handle_opponents(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> Option<RouteTarget> {
        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::Tab)) {
            let selected_opponents = self.selected_opponents.clone();
            self.update_config(
                &mut state.config,
                move |config| commit_opponents(config, &selected_opponents),
                cx,
            );
            self.mode = KothMode::Main;
            return None;
        }

        let max_idx = self
            .resources
            .player_names(state.config.name_set_index as usize)
            .len();
        if max_idx == 0 {
            return None;
        }

        match event {
            UiEvent::KeyDown(Key::Down) | UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                let prev_idx = self.preview_opponent;
                if self.selected_opponents.len() < 20
                    && !self.selected_opponents.contains(&prev_idx)
                {
                    self.selected_opponents.push(prev_idx);
                } else if self.selected_opponents.contains(&prev_idx) {
                    state.request_beep(Beep::Type2);
                }

                let mut next = (prev_idx + 1) % max_idx;
                for _ in 0..max_idx {
                    if !self.selected_opponents.contains(&next) {
                        self.preview_opponent = next;
                        break;
                    }
                    next = (next + 1) % max_idx;
                }
                None
            }
            UiEvent::KeyDown(Key::Up) | UiEvent::KeyDown(Key::Backspace) => {
                if let Some(idx) = self.selected_opponents.pop() {
                    self.preview_opponent = idx;
                }
                None
            }
            UiEvent::KeyDown(Key::Left) => {
                self.step_preview_opponent(max_idx, -1);
                None
            }
            UiEvent::KeyDown(Key::Right) => {
                self.step_preview_opponent(max_idx, 1);
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

    fn apply_pack(&mut self, pack: i32, state: &mut GameState, cx: &Persistence) {
        self.update_config(
            &mut state.config,
            |config| builder::apply_koth_pack(config, pack as u8),
            cx,
        );
        self.mode = KothMode::Main;
    }

    fn activate(
        &mut self,
        n: usize,
        state: &mut GameState,
        cx: &Persistence,
    ) -> Option<RouteTarget> {
        match n {
            0 => Some(RouteTarget::MainMenu),
            1 => self.start_koth(state),
            2 => {
                let pack = state.config.koth_pack;
                self.pack_cursor = if pack == 0 {
                    6
                } else {
                    (pack as usize).saturating_sub(1).min(5)
                };
                self.mode = KothMode::Packs;
                None
            }
            3 => {
                self.update_config(&mut state.config, |cfg| cfg.koth_pack = 0, cx);
                let names_len = self
                    .resources
                    .player_names(state.config.name_set_index as usize)
                    .len();
                let configured = if state.config.koth_opponent_count > 0 {
                    state
                        .config
                        .koth_opponent_ids
                        .iter()
                        .filter_map(|&v| usize::try_from(v).ok())
                        .filter(|&idx| idx < names_len)
                        .collect()
                } else {
                    Vec::new()
                };
                self.selected_opponents = unique_opponents(&configured);
                self.preview_opponent = first_unselected(&self.selected_opponents, names_len);
                self.mode = KothMode::Opponents;
                None
            }
            4 => Some(RouteTarget::KothHillPicker),
            5 => {
                self.update_config(
                    &mut state.config,
                    |cfg| cfg.koth_wind = if cfg.koth_wind != 0 { 0 } else { 1 },
                    cx,
                );
                None
            }
            6 => {
                self.update_config(
                    &mut state.config,
                    |cfg| {
                        cfg.koth_rounds = if cfg.koth_rounds == 1 { 2 } else { 1 };
                    },
                    cx,
                );
                None
            }
            _ => None,
        }
    }

    fn start_koth(&self, state: &mut GameState) -> Option<RouteTarget> {
        let profiles = state.profiles.clone();
        let hill_count = self.resources.hills.len();
        let random_hill_count = self.resources.hills.original_count();
        let name_set_index = state.config.name_set_index as usize;
        let rng_clone = state.rng.clone();
        let comp = factory::koth(
            &state.config,
            &profiles,
            self.resources.player_names(name_set_index),
            hill_count,
            random_hill_count,
            state.config.unique_computer_names != 0,
            rng_clone,
        );
        state.start_active(comp);
        Some(RouteTarget::CompetitionJump)
    }
}

fn pack_cursor_y(cur: usize) -> i32 {
    120 + cur as i32 * 8 + if cur == 6 { 8 } else { 0 }
}

fn pack_for_row(row: usize) -> u8 {
    if row == 6 {
        0
    } else {
        row as u8 + 1
    }
}

fn koth_pack_title(pack: u8, lang: &LangBase) -> String {
    match pack {
        1 => format!("1. {}", lang.tr(131)),
        2 => format!("2. {}", lang.tr(132)),
        3 => format!("3. {}", lang.tr(133)),
        4 => format!("4. {}", lang.tr(134)),
        5 => format!("5. {}", lang.tr(135)),
        6 => format!("6. {}", lang.tr(136)),
        0 => format!("0. {}", lang.tr(137)),
        _ => format!("{pack}. ?"),
    }
}

fn commit_opponents(config: &mut Config, selected_opponents: &[usize]) {
    config.koth_opponent_ids = unique_opponents(selected_opponents)
        .iter()
        .map(|&idx| idx as i32)
        .collect();
    config.koth_opponent_count = config.koth_opponent_ids.len() as i32;
    config.koth_pack = 0;
}

fn first_unselected(selected: &[usize], total: usize) -> usize {
    (0..total).find(|idx| !selected.contains(idx)).unwrap_or(0)
}

fn unique_opponents(opponents: &[usize]) -> Vec<usize> {
    let mut unique = Vec::with_capacity(opponents.len().min(20));
    for &opponent in opponents.iter().take(20) {
        if !unique.contains(&opponent) {
            unique.push(opponent);
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committing_opponents_preserves_ids_and_updates_count() {
        let mut config = Config::default();

        commit_opponents(&mut config, &[7, 2, 11]);

        assert_eq!(config.koth_opponent_ids, vec![7, 2, 11]);
        assert_eq!(config.koth_opponent_count, 3);
        assert_eq!(config.koth_pack, 0);
    }

    #[test]
    fn committing_no_opponents_clears_previous_selection() {
        let mut config = Config::default();

        commit_opponents(&mut config, &[]);

        assert!(config.koth_opponent_ids.is_empty());
        assert_eq!(config.koth_opponent_count, 0);
        assert_eq!(config.koth_pack, 0);
    }

    #[test]
    fn pack_rows_map_labels_to_the_same_pack_and_custom_last() {
        assert_eq!(
            (0..7).map(pack_for_row).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6, 0]
        );
        assert_eq!(
            (0..=6).map(pack_cursor_y).collect::<Vec<_>>(),
            [120, 128, 136, 144, 152, 160, 176]
        );
    }
}
