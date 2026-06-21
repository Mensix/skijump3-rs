use crate::data::records::{HillRecord, Hiscore};
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_GREEN, BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::{format_decimal, ordinal_dot};
use crate::text::layout::{is_computer_name, shorten_name};
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent};

const HALL_PAGES: usize = 3;
const PAGE_SIZE: usize = 20;

#[derive(Debug)]
pub struct HallOfFameView {
    resources: ResourcesRef,
    page: usize,
}

impl HallOfFameView {
    pub const fn new(resources: ResourcesRef) -> Self {
        Self { resources, page: 0 }
    }

    fn paint_content(&self, state: &GameState, cx: &mut PaintCx<'_>) {
        if self.page >= 2 {
            paint_screen(cx, 1, BG_GREEN);
        } else {
            match self.page {
                1 => paint_screen(cx, 4, BG_PURPLE),
                _ => paint_screen(cx, 1, BG_PURPLE),
            };
        }

        match self.page {
            0 => self.paint_list(state, cx, 0),
            1 => {
                self.paint_list(state, cx, 1);
                self.paint_list(state, cx, 2);
            }
            _ => self.paint_koth_records(state, cx),
        }

        paint_page_hints(
            cx,
            self.page,
            HALL_PAGES,
            &self.resources.langbase.tr(246),
            &self.resources.langbase.tr(247),
            &self.resources.langbase.tr(248),
        );
    }

    fn paint_list(&self, state: &GameState, cx: &mut PaintCx<'_>, phase: usize) {
        let mut yy = 6;
        let col = [30, 146, 173, 215];
        let (title, entries, start, sortby) = match phase {
            0 => (
                self.resources.langbase.tr(163),
                20,
                1,
                false,
            ),
            1 => (
                self.resources.langbase.tr(164),
                10,
                21,
                false,
            ),
            _ => {
                yy = 126;
                (
                    self.resources.langbase.tr(165),
                    5,
                    31,
                    true,
                )
            }
        };

        cx.text((30, yy), FONT_BODY, title);
        yy += 17;

        cx.text(
            (col[0], yy),
            FONT_GOLD,
            self.resources.langbase.tr(166),
        );
        cx.text(
            (col[1], yy),
            FONT_GOLD,
            self.resources.langbase.tr(167),
        );
        cx.text(
            (col[2], yy),
            FONT_GOLD,
            self.resources.langbase.tr(168),
        );
        cx.text(
            (col[3], yy),
            FONT_GOLD,
            self.resources.langbase.tr(169),
        );

        let records = &state.records;
        for idx in start..start + entries {
            yy += 8;
            let Some(hi) = records.top(idx) else {
                continue;
            };
            self.paint_hiscore_row(cx, hi, idx - start + 1, yy, col, sortby);
        }
    }

    fn paint_hiscore_row(
        &self,
        cx: &mut PaintCx<'_>,
        hi: &Hiscore,
        place: usize,
        y: i32,
        col: [i32; 4],
        sortby_points: bool,
    ) {
        let name_color = if is_computer_name(&hi.name) {
            FONT_TEAL
        } else {
            FONT_BODY
        };
        cx.right_text((24, y), FONT_GOLD, ordinal_dot(place));
        cx.text(
            (col[0], y),
            name_color,
            shorten_name(&hi.name, &self.resources.font, 110),
        );
        cx.right_text((col[1] + 14, y), name_color, ordinal_dot(hi.pos));
        let score = if sortby_points {
            format_decimal(hi.score)
        } else {
            format!("{:.0}", hi.score)
        };
        cx.right_text((col[2] + 24, y), name_color, score);
        cx.text((col[3], y), FONT_GRAY, &hi.time);
    }

    fn paint_koth_records(&self, state: &GameState, cx: &mut PaintCx<'_>) {
        let col = [30, 55, 175, 290];
        let mut yy = 12;
        cx.text(
            (30, 6),
            FONT_BODY,
            self.resources.langbase.tr(160),
        );

        let records = &state.records;
        for idx in 0..6 {
            yy += 18;
            cx.text(
                (col[0], yy),
                FONT_GOLD,
                format!(
                    "{}. {}",
                    idx + 1,
                    self.resources.langbase.tr(131 + idx)
                ),
            );
            yy += 10;

            let name = self.resources.langbase.tr(161);
            let Some(hi) = records.top(idx + 36) else {
                cx.text((col[1], yy), FONT_GRAY, name);
                continue;
            };
            if hi.score > 0.0 {
                cx.text((col[2], yy), FONT_GRAY, &hi.time);
                cx.text((col[3], yy), FONT_BODY, format!("{:.0} X", hi.score));
                cx.text((col[1], yy), FONT_BODY, &hi.name);
            } else {
                cx.text((col[1], yy), FONT_GRAY, name);
            }
        }
    }
}

impl GameScreen for HallOfFameView {
    fn event(
        &mut self,
        _cx: &mut GameCx<'_>,
        nav: &mut ScreenEventCx<RouteTarget>,
        event: UiEvent,
    ) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if let Some(route) = handle_paged_ui_event(event, &mut self.page, HALL_PAGES) {
            nav.navigate(route);
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(cx.state, paint);
    }
}

#[derive(Debug)]
pub struct HillRecordsView {
    resources: ResourcesRef,
    page: usize,
}

impl HillRecordsView {
    pub const fn new(resources: ResourcesRef) -> Self {
        Self { resources, page: 0 }
    }

    fn pages(&self) -> usize {
        self.resources.hills.len().div_ceil(PAGE_SIZE).max(1)
    }

    fn paint_content(&self, state: &GameState, cx: &mut PaintCx<'_>) {
        let pages = self.pages();
        paint_screen(cx, 1, BG_PURPLE);
        self.paint_hill_records(state, cx);
        paint_page_hints(
            cx,
            self.page,
            pages,
            &self.resources.langbase.tr(246),
            &self.resources.langbase.tr(247),
            &self.resources.langbase.tr(248),
        );
    }

    fn paint_hill_records(&self, state: &GameState, cx: &mut PaintCx<'_>) {
        let col = [3, 71, 183, 200, 216];
        let phase = self.page;
        let start = phase * PAGE_SIZE;
        let loop_count = (self.resources.hills.len().saturating_sub(start)).min(PAGE_SIZE);
        let title = if phase == 0 {
            self.resources.langbase.tr(170)
        } else {
            self.resources.langbase.tr(156)
        };
        cx.text((30, 6), FONT_BODY, title);
        cx.text(
            (col[0], 23),
            FONT_BODY,
            self.resources.langbase.tr(106),
        );
        cx.text(
            (col[1], 23),
            FONT_BODY,
            self.resources.langbase.tr(171),
        );
        cx.right_text(
            (col[2], 23),
            FONT_BODY,
            self.resources.langbase.tr(172),
        );
        cx.text((col[3], 23), FONT_BODY, "(K)");
        cx.text(
            (col[4], 23),
            FONT_BODY,
            self.resources.langbase.tr(169),
        );

        let records = &state.records;
        let mut ahi_sum = 0.0;
        for aa in 0..loop_count {
            let idx = aa + start;
            let y = (aa as i32) * 8 + 32;
            let Some(hill) = self.resources.hills.hill(idx) else {
                continue;
            };
            let record = records.hill_record(idx).cloned().unwrap_or_default();
            let display_len = ahi_len(&record, hill.kr);
            if phase == 0 {
                ahi_sum += display_len;
            }

            cx.text(
                (col[0], y),
                FONT_GOLD,
                shorten_name(&hill.name, &self.resources.font, 64),
            );
            let record_color = if is_computer_name(&record.name) {
                FONT_TEAL
            } else {
                FONT_BODY
            };
            cx.text(
                (col[1], y),
                record_color,
                shorten_name(&record.name, &self.resources.font, 80),
            );
            let length_color = if is_computer_name(&record.name) {
                FONT_TEAL
            } else {
                FONT_GOLD
            };
            cx.right_text((col[2], y), length_color, format_decimal(record.len));
            cx.right_text((col[3] + 11, y), length_color, format!("({})", hill.kr));
            cx.text((col[4], y), FONT_GRAY, record.time);
        }

        if phase == 0 {
            let total: f64 = (0..self.resources.hills.len().min(PAGE_SIZE))
                .filter_map(|idx| self.resources.hills.hill(idx).map(|hill| hill.kr as f64))
                .sum();
            if total > 0.0 {
                cx.text((130, 192), FONT_GRAY, "A.H.I.");
                cx.right_text((197, 192), FONT_GRAY, format_ahi(ahi_sum, total));
            }
        }
    }
}

fn ahi_len(record: &HillRecord, hill_kr: i64) -> f64 {
    if is_computer_name(&record.name) {
        hill_kr as f64
    } else {
        record.len
    }
}

fn format_ahi(sum: f64, total: f64) -> String {
    let value = ((sum / total) * 1000.0).round() as i64;
    let mut out = value.to_string();
    let pos = out.len().saturating_sub(1);
    out.insert(pos, '.');
    format!("{out} %")
}

impl GameScreen for HillRecordsView {
    fn event(
        &mut self,
        _cx: &mut GameCx<'_>,
        nav: &mut ScreenEventCx<RouteTarget>,
        event: UiEvent,
    ) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        let pages = self.pages();
        if let Some(route) = handle_paged_ui_event(event, &mut self.page, pages) {
            nav.navigate(route);
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(cx.state, paint);
    }
}

fn paint_screen(cx: &mut PaintCx<'_>, style: u8, bg: engine::color::Rgba) {
    cx.fill((0, 0, 320, 200), BLACK);
    match style {
        1 => {
            cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
            cx.fill((0, 20, 320, 180), bg);
        }
        4 => {
            cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
            cx.fill((0, 20, 320, 99), bg);
            cx.pattern_fill((0, 120, 320, 19), FILL_GRAY);
            cx.fill((0, 140, 320, 60), bg);
        }
        _ => {}
    }
    match style {
        1 => cx.sprite(sprites::Sprite::Logo as u16, (5, 2)),
        4 => {
            cx.sprite(sprites::Sprite::Logo as u16, (5, 2));
            cx.sprite(sprites::Sprite::Logo as u16, (5, 122));
        }
        _ => {}
    }
}

fn paint_page_hints(
    cx: &mut PaintCx<'_>,
    page: usize,
    pages: usize,
    prev: &str,
    next: &str,
    end: &str,
) {
    if page > 0 {
        cx.right_text((319, 5), FONT_GRAY, format!("(-{prev}"));
    }
    let text = if page + 1 == pages { end } else { next };
    cx.right_text((319, 13), FONT_GRAY, format!("{text}-)"));
}

fn handle_paged_ui_event(event: UiEvent, page: &mut usize, pages: usize) -> Option<RouteTarget> {
    if *page >= pages {
        *page = pages.saturating_sub(1);
    }
    match event {
        UiEvent::KeyDown(Key::Escape) => Some(RouteTarget::Back),
        UiEvent::KeyDown(Key::Home) => {
            *page = 0;
            None
        }
        UiEvent::KeyDown(Key::Left | Key::PageUp) if *page > 0 => {
            *page = (*page).saturating_sub(1);
            None
        }
        UiEvent::KeyDown(Key::Right | Key::PageDown | Key::Enter) | UiEvent::Text(' ') => {
            *page += 1;
            if *page >= pages {
                Some(RouteTarget::MainMenu)
            } else {
                None
            }
        }
        UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => None,
    }
}
