use crate::data::records::{HillRecord, Hiscore};
use crate::gfx::palette::{
    BG_KOTH, BG_LEFT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GREET, FONT_HELP, FONT_NEW,
};
use crate::gfx::sprites;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::{format_decimal, ordinal_dot};
use crate::text::layout::{is_computer_name, lstr, shorten_name};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::oxide::input::Key;

const HALL_PAGES: usize = 3;
const PAGE_SIZE: usize = 20;

#[derive(Debug)]
pub struct HallOfFameView {
    resources: ResourcesRef,
    store: StoreRef,
    page: usize,
}

impl HallOfFameView {
    pub const fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            resources,
            store,
            page: 0,
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        if self.page >= 2 {
            paint_screen(cx, 1, BG_KOTH);
        } else {
            match self.page {
                1 => paint_screen(cx, 4, BG_LEFT),
                _ => paint_screen(cx, 1, BG_LEFT),
            };
        }

        match self.page {
            0 => self.paint_list(cx, 0),
            1 => {
                self.paint_list(cx, 1);
                self.paint_list(cx, 2);
            }
            _ => self.paint_koth_records(cx),
        }

        paint_page_hints(
            cx,
            self.page,
            HALL_PAGES,
            &lstr(&self.resources.langbase, 246, "Back"),
            &lstr(&self.resources.langbase, 247, "Next"),
            &lstr(&self.resources.langbase, 248, "End"),
        );
    }

    fn paint_list(&self, cx: &mut PaintCx<'_>, phase: usize) {
        let mut yy = 6;
        let col = [30, 146, 173, 215];
        let (title, entries, start, sortby) = match phase {
            0 => (
                lstr(&self.resources.langbase, 163, "World Cup"),
                20,
                1,
                false,
            ),
            1 => (
                lstr(&self.resources.langbase, 164, "Team Cup"),
                10,
                21,
                false,
            ),
            _ => {
                yy = 126;
                (
                    lstr(&self.resources.langbase, 165, "Four Hills"),
                    5,
                    31,
                    true,
                )
            }
        };

        cx.text((30, yy), FONT_DEFAULT, title);
        yy += 17;

        cx.text(
            (col[0], yy),
            FONT_NEW,
            lstr(&self.resources.langbase, 166, "Name"),
        );
        cx.text(
            (col[1], yy),
            FONT_NEW,
            lstr(&self.resources.langbase, 167, "Pos"),
        );
        cx.text(
            (col[2], yy),
            FONT_NEW,
            lstr(&self.resources.langbase, 168, "Points"),
        );
        cx.text(
            (col[3], yy),
            FONT_NEW,
            lstr(&self.resources.langbase, 169, "Date"),
        );

        let records = self.store.records();
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
            FONT_GREET
        } else {
            FONT_DEFAULT
        };
        cx.right_text((24, y), FONT_NEW, ordinal_dot(place));
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
        cx.text((col[3], y), FONT_HELP, &hi.time);
    }

    fn paint_koth_records(&self, cx: &mut PaintCx<'_>) {
        let col = [30, 55, 175, 290];
        let mut yy = 12;
        cx.text(
            (30, 6),
            FONT_DEFAULT,
            lstr(&self.resources.langbase, 160, "King of the Hill"),
        );

        let records = self.store.records();
        for idx in 1..=6 {
            yy += 18;
            cx.text(
                (col[0], yy),
                FONT_NEW,
                format!(
                    "{}. {}",
                    idx,
                    lstr(&self.resources.langbase, 130 + idx, "Challenge")
                ),
            );
            yy += 10;

            let name = lstr(&self.resources.langbase, 161, "Nobody");
            let Some(hi) = records.top(idx + 35) else {
                cx.text((col[1], yy), FONT_HELP, name);
                continue;
            };
            if hi.score > 0.0 {
                cx.text((col[2], yy), FONT_HELP, &hi.time);
                cx.text((col[3], yy), FONT_DEFAULT, format!("{:.0} X", hi.score));
                cx.text((col[1], yy), FONT_DEFAULT, &hi.name);
            } else {
                cx.text((col[1], yy), FONT_HELP, name);
            }
        }
    }
}

impl Screen<RouteTarget> for HallOfFameView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if let Some(route) = handle_paged_ui_event(event, &mut self.page, HALL_PAGES) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

#[derive(Debug)]
pub struct HillRecordsView {
    resources: ResourcesRef,
    store: StoreRef,
    page: usize,
}

impl HillRecordsView {
    pub const fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            resources,
            store,
            page: 0,
        }
    }

    fn pages(&self) -> usize {
        self.resources.hills.len().div_ceil(PAGE_SIZE).max(1)
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        let pages = self.pages();
        paint_screen(cx, 1, BG_LEFT);
        self.paint_hill_records(cx);
        paint_page_hints(
            cx,
            self.page,
            pages,
            &lstr(&self.resources.langbase, 246, "Back"),
            &lstr(&self.resources.langbase, 247, "Next"),
            &lstr(&self.resources.langbase, 248, "End"),
        );
    }

    fn paint_hill_records(&self, cx: &mut PaintCx<'_>) {
        let col = [3, 71, 183, 200, 216];
        let phase = self.page;
        let start = phase * PAGE_SIZE;
        let loop_count = (self.resources.hills.len().saturating_sub(start)).min(PAGE_SIZE);
        let title = if phase == 0 {
            lstr(&self.resources.langbase, 170, "Hill Records")
        } else {
            lstr(&self.resources.langbase, 156, "Extra Hill Records")
        };
        cx.text((30, 6), FONT_DEFAULT, title);
        cx.text(
            (col[0], 23),
            FONT_DEFAULT,
            lstr(&self.resources.langbase, 106, "Hill"),
        );
        cx.text(
            (col[1], 23),
            FONT_DEFAULT,
            lstr(&self.resources.langbase, 171, "Who"),
        );
        cx.right_text(
            (col[2], 23),
            FONT_DEFAULT,
            lstr(&self.resources.langbase, 172, "Length"),
        );
        cx.text((col[3], 23), FONT_DEFAULT, "(K)");
        cx.text(
            (col[4], 23),
            FONT_DEFAULT,
            lstr(&self.resources.langbase, 169, "Date"),
        );

        let records = self.store.records();
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
                FONT_NEW,
                shorten_name(&hill.name, &self.resources.font, 64),
            );
            let record_color = if is_computer_name(&record.name) {
                FONT_GREET
            } else {
                FONT_DEFAULT
            };
            cx.text(
                (col[1], y),
                record_color,
                shorten_name(&record.name, &self.resources.font, 80),
            );
            let length_color = if is_computer_name(&record.name) {
                FONT_GREET
            } else {
                FONT_NEW
            };
            cx.right_text((col[2], y), length_color, format_decimal(record.len));
            cx.right_text((col[3] + 11, y), length_color, format!("({})", hill.kr));
            cx.text((col[4], y), FONT_HELP, record.time);
        }

        if phase == 0 {
            let total: f64 = (0..self.resources.hills.len().min(PAGE_SIZE))
                .filter_map(|idx| self.resources.hills.hill(idx).map(|hill| hill.kr as f64))
                .sum();
            if total > 0.0 {
                cx.text((130, 192), FONT_HELP, "A.H.I.");
                cx.right_text((197, 192), FONT_HELP, format_ahi(ahi_sum, total));
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

impl Screen<RouteTarget> for HillRecordsView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        let pages = self.pages();
        if let Some(route) = handle_paged_ui_event(event, &mut self.page, pages) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn paint_screen(cx: &mut PaintCx<'_>, style: u8, bg: engine::color::Rgba) {
    cx.fill((0, 0, 320, 200), BLACK);
    match style {
        1 => {
            cx.fill((0, 0, 320, 19), FILL_DIM);
            cx.fill((0, 20, 320, 180), bg);
        }
        4 => {
            cx.fill((0, 0, 320, 19), FILL_DIM);
            cx.fill((0, 20, 320, 99), bg);
            cx.fill((0, 120, 320, 19), FILL_DIM);
            cx.fill((0, 140, 320, 60), bg);
        }
        _ => {}
    }
    cx.dither_fill(63);
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
        cx.right_text((319, 5), FONT_HELP, format!("(-{prev}"));
    }
    let text = if page + 1 == pages { end } else { next };
    cx.right_text((319, 13), FONT_HELP, format!("{text}-)"));
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
