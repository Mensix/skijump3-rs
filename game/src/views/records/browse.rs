use crate::competition::types::CustomCupScoring;
use crate::components::page_nav::{
    page_event, render_page_hints, PageCursor, PageDismissal, PageEventMap, PageHintLayout,
};
use crate::data::records::{HillRecord, Hiscore};
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_GREEN, BG_PURPLE, BLACK, FILL_GRAY, FILL_TEAL, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::save::custom_cup::{discover_custom_cup_files, CustomCupFile};
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::{format_decimal, format_time, ordinal_dot};
use crate::text::layout::shorten_name;
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenEventCx, UiEvent};

const PAGE_SIZE: usize = 20;

#[derive(Debug)]
pub struct HallOfFameView {
    resources: ResourcesRef,
    page: PageCursor,
    custom_cups: Vec<(String, CustomCupFile)>,
}

impl HallOfFameView {
    pub fn new(resources: ResourcesRef) -> Self {
        let custom_cups = discover_custom_cup_files(&resources.files);
        Self {
            resources,
            page: PageCursor::new(),
            custom_cups,
        }
    }

    fn pages(&self) -> usize {
        cup_pages(self.custom_cups.len())
    }

    fn paint_content(&self, state: &GameState, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let page = self.page.current();
        if page == 2 {
            paint_screen(cx, 1, BG_GREEN);
        } else {
            match page {
                1 => paint_screen(cx, 4, BG_PURPLE),
                _ => paint_screen(cx, 1, BG_PURPLE),
            };
        }

        match page {
            0 => self.paint_list(state, cx, 0),
            1 => {
                self.paint_list(state, cx, 1);
                self.paint_list(state, cx, 2);
            }
            2 => self.paint_koth_records(state, cx),
            _ => self.paint_custom_cup_records(cx),
        }

        render_page_hints(cx, page, self.pages(), lang, PageHintLayout::Top);
    }

    fn paint_custom_cup_records(&self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let col = [50, 157, 184, 215];
        let start = self.page.current().saturating_sub(3) * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.custom_cups.len());
        cx.text((30, 6), FONT_BODY, lang.tr(162));
        cx.text((5, 23), FONT_GOLD, lang.tr(173));
        cx.text((col[0], 23), FONT_GOLD, lang.tr(171));
        cx.right_text((col[1] + 10, 23), FONT_GOLD, lang.tr(167));
        cx.right_text((col[2] + 24, 23), FONT_GOLD, lang.tr(168));
        cx.text((col[3], 23), FONT_GOLD, lang.tr(169));

        for (row, (name, cup)) in self.custom_cups[start..end].iter().enumerate() {
            let record = best_custom_cup_record(cup);
            let y = 32 + row as i32 * 8;
            cx.text((5, y), FONT_GRAY, name);
            let Some(record) = record else {
                continue;
            };
            let name_color = if record.is_computer {
                FONT_TEAL
            } else {
                FONT_BODY
            };
            cx.text(
                (col[0], y),
                name_color,
                &shorten_name(&record.name, &self.resources.font, 98),
            );
            cx.right_text((col[1] + 10, y), name_color, &ordinal_dot(record.pos));
            let score = if matches!(cup.scoring, CustomCupScoring::AggregateJumpPoints) {
                format_decimal(record.score)
            } else {
                format!("{:.0}", record.score)
            };
            cx.right_text((col[2] + 24, y), name_color, &score);
            cx.text((col[3], y), FONT_GRAY, &format_time(&record.time));
        }
    }

    fn paint_list(&self, state: &GameState, cx: &mut dyn UiCanvas, phase: usize) {
        let lang = &self.resources.langbase;
        let mut yy = 6;
        let col = [30, 146, 173, 215];
        let (title, entries, start, sortby) = match phase {
            0 => (lang.tr(163), 20, 0, false),
            1 => (lang.tr(164), 10, 20, false),
            _ => {
                yy = 126;
                (lang.tr(165), 5, 30, true)
            }
        };

        cx.text((30, yy), FONT_BODY, title);
        yy += 17;

        cx.text((col[0], yy), FONT_GOLD, lang.tr(166));
        cx.text((col[1], yy), FONT_GOLD, lang.tr(167));
        cx.text((col[2], yy), FONT_GOLD, lang.tr(168));
        cx.text((col[3], yy), FONT_GOLD, lang.tr(169));

        let records = &state.records;
        for idx in start..start + entries {
            yy += 8;
            let hi = records.top(idx).cloned().unwrap_or_default();
            self.paint_hiscore_row(cx, &hi, idx - start + 1, yy, col, sortby);
        }
    }

    fn paint_hiscore_row(
        &self,
        cx: &mut dyn UiCanvas,
        hi: &Hiscore,
        place: usize,
        y: i32,
        col: [i32; 4],
        sortby_points: bool,
    ) {
        let name_color = if hi.is_computer { FONT_TEAL } else { FONT_BODY };
        cx.right_text((24, y), FONT_GOLD, &ordinal_dot(place));
        cx.text(
            (col[0], y),
            name_color,
            &shorten_name(&hi.name, &self.resources.font, 110),
        );
        cx.right_text((col[1] + 14, y), name_color, &ordinal_dot(hi.pos));
        let score = if sortby_points {
            format_decimal(hi.score)
        } else {
            format!("{:.0}", hi.score)
        };
        cx.right_text((col[2] + 24, y), name_color, &score);
        cx.text((col[3], y), FONT_GRAY, &format_time(&hi.time));
    }

    fn paint_koth_records(&self, state: &GameState, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let col = [30, 55, 175, 290];
        let mut yy = 12;
        cx.text((30, 6), FONT_BODY, lang.tr(160));

        let records = &state.records;
        for idx in 0..6 {
            yy += 18;
            cx.text(
                (col[0], yy),
                FONT_GOLD,
                &format!("{}. {}", idx + 1, lang.tr(131 + idx)),
            );
            yy += 10;

            let name = lang.tr(161);
            let Some(hi) = records.top(idx + 35) else {
                cx.text((col[1], yy), FONT_GRAY, name);
                continue;
            };
            if hi.score > 0.0 {
                cx.text((col[2], yy), FONT_GRAY, &format_time(&hi.time));
                cx.text((col[3], yy), FONT_BODY, &format!("{:.0} X", hi.score));
                cx.text((col[1], yy), FONT_BODY, &hi.name);
            } else {
                cx.text((col[1], yy), FONT_GRAY, name);
            }
        }
    }
}

fn best_custom_cup_record(cup: &CustomCupFile) -> Option<&Hiscore> {
    cup.records.iter().find(|record| !record.name.is_empty())
}

impl GameScreen for HallOfFameView {
    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            nav.back();
            return;
        }
        let pages = self.pages();
        if handle_paged_ui_event(event, &mut self.page, pages) {
            nav.navigate(RouteTarget::MainMenu);
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(cx.state, paint);
    }
}

#[derive(Debug)]
pub struct HillRecordsView {
    resources: ResourcesRef,
    page: PageCursor,
}

impl HillRecordsView {
    pub fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            page: PageCursor::default(),
        }
    }

    fn pages(&self) -> usize {
        self.resources.hills.len().div_ceil(PAGE_SIZE).max(1)
    }

    fn paint_content(&self, state: &GameState, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let pages = self.pages();
        let background = if self.page.current() == 0 {
            BG_PURPLE
        } else {
            FILL_TEAL
        };
        paint_screen(cx, 1, background);
        self.paint_hill_records(state, cx);
        render_page_hints(cx, self.page.current(), pages, lang, PageHintLayout::Top);
    }

    fn paint_hill_records(&self, state: &GameState, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let col = [3, 71, 183, 200, 216];
        let phase = self.page.current();
        let start = phase * PAGE_SIZE;
        let loop_count = (self.resources.hills.len().saturating_sub(start)).min(PAGE_SIZE);
        let title = if phase == 0 {
            lang.tr(170)
        } else {
            lang.tr(156)
        };
        cx.text((30, 6), FONT_BODY, title);
        cx.text((col[0], 23), FONT_BODY, lang.tr(106));
        cx.text((col[1], 23), FONT_BODY, lang.tr(171));
        cx.right_text((col[2], 23), FONT_BODY, lang.tr(172));
        cx.text((col[3], 23), FONT_BODY, "(K)");
        cx.text((col[4], 23), FONT_BODY, lang.tr(169));

        let records = &state.records;
        let mut ahi_sum = 0.0;
        for aa in 0..loop_count {
            let idx = aa + start;
            let y = (aa as i32) * 8 + 32;
            let Some(hill) = self.resources.hills.hill(idx) else {
                continue;
            };
            let record = records
                .hill_record(&hill.record_key)
                .cloned()
                .unwrap_or_default();
            let display_len = ahi_len(&record, hill.kr);
            if phase == 0 {
                ahi_sum += display_len;
            }

            cx.text(
                (col[0], y),
                FONT_GOLD,
                &shorten_name(&hill.name, &self.resources.font, 64),
            );
            let record_color = if record.is_computer {
                FONT_TEAL
            } else {
                FONT_BODY
            };
            cx.text(
                (col[1], y),
                record_color,
                &shorten_name(&record.name, &self.resources.font, 80),
            );
            let length_color = if record.is_computer {
                FONT_TEAL
            } else {
                FONT_GOLD
            };
            cx.right_text((col[2], y), length_color, &format_decimal(record.len));
            cx.right_text((col[3] + 11, y), length_color, &format!("({})", hill.kr));
            cx.text((col[4], y), FONT_GRAY, &format_time(&record.time));
        }

        if phase == 0 {
            let total: f64 = (0..self.resources.hills.len().min(PAGE_SIZE))
                .filter_map(|idx| self.resources.hills.hill(idx).map(|hill| hill.kr as f64))
                .sum();
            if total > 0.0 {
                cx.text((130, 192), FONT_GRAY, "A.H.I.");
                cx.right_text((197, 192), FONT_GRAY, &format_ahi(ahi_sum, total));
            }
        }
    }
}

fn ahi_len(record: &HillRecord, hill_kr: i64) -> f64 {
    if record.is_computer {
        hill_kr as f64
    } else {
        record.len
    }
}

fn format_ahi(sum: f64, total: f64) -> String {
    let value = ((sum / total) * 10000.0).round() as i64;
    let mut out = value.to_string();
    let pos = out.len().saturating_sub(2);
    out.insert(pos, '.');
    format!("{out} %")
}

impl GameScreen for HillRecordsView {
    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            nav.back();
            return;
        }
        let pages = self.pages();
        if handle_paged_ui_event(event, &mut self.page, pages) {
            nav.navigate(RouteTarget::MainMenu);
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(cx.state, paint);
    }
}

fn paint_screen(cx: &mut dyn UiCanvas, style: u8, bg: engine::color::Rgba) {
    cx.fill((0, 0, 320, 200), BLACK);
    match style {
        1 => {
            cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
            cx.pattern_fill((0, 20, 320, 180), bg);
        }
        4 => {
            cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
            cx.pattern_fill((0, 20, 320, 99), bg);
            cx.pattern_fill((0, 120, 320, 19), FILL_GRAY);
            cx.pattern_fill((0, 140, 320, 60), bg);
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

fn cup_pages(cup_count: usize) -> usize {
    3 + cup_count.div_ceil(PAGE_SIZE)
}

fn handle_paged_ui_event(event: UiEvent, page: &mut PageCursor, pages: usize) -> bool {
    page_event(event, PageEventMap::Records)
        .is_some_and(|event| page.apply(event, pages, PageDismissal::RECORDS))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cup(records: Vec<Hiscore>) -> CustomCupFile {
        CustomCupFile {
            format_version: 1,
            hill_refs: vec![],
            scoring: CustomCupScoring::WorldCupPoints,
            records,
        }
    }

    fn record(name: &str, score: f64) -> Hiscore {
        Hiscore {
            name: name.into(),
            score,
            ..Hiscore::default()
        }
    }

    #[test]
    fn custom_cups_are_grouped_into_pages_of_twenty_files() {
        assert_eq!(cup_pages(0), 3);
        assert_eq!(cup_pages(20), 4);
        assert_eq!(cup_pages(21), 5);
        assert_eq!(cup_pages(43), 6);
    }

    #[test]
    fn custom_cup_hall_of_fame_uses_only_the_best_nonempty_record() {
        let cup = cup(vec![
            record("BEST", 500.0),
            record("OTHER", 400.0),
            Hiscore::default(),
        ]);

        let best = best_custom_cup_record(&cup).unwrap();

        assert_eq!(best.name, "BEST");
        assert_eq!(best.score, 500.0);
    }

    #[test]
    fn custom_cup_without_records_has_no_hall_of_fame_entry() {
        assert!(best_custom_cup_record(&cup(vec![])).is_none());
        assert!(best_custom_cup_record(&cup(vec![Hiscore::default()])).is_none());
    }
}
