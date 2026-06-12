use crate::components::page_nav;
use crate::components::screen::{new_screen, new_screen_with_bg, page_hints};
use crate::data::records::{HillRecord, Hiscore};
use crate::gfx::palette::{BG_KOTH, FONT_DEFAULT, FONT_GREET, FONT_HELP, FONT_NEW};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::{format_decimal, ordinal_dot};
use crate::text::layout::{is_computer_name, lstr, shorten_name};
use engine::oxide::legacy::{commands_to_elements, event_from_ui, paint_elements};
use engine::oxide::{CommandBuffer, NavAction, PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Cell, Table};
use engine::ui::{Element, Event, View};

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

    fn legacy_elements(&self) -> Vec<Element> {
        let mut els = if self.page >= 2 {
            new_screen_with_bg(1, BG_KOTH)
        } else {
            match self.page {
                1 => new_screen(4),
                _ => new_screen(1),
            }
        };

        match self.page {
            0 => self.draw_list(&mut els, 0),
            1 => {
                self.draw_list(&mut els, 1);
                self.draw_list(&mut els, 2);
            }
            _ => self.draw_koth_records(&mut els),
        }

        els.extend(page_hints(
            self.page,
            HALL_PAGES,
            &lstr(&self.resources.langbase, 246, "Back"),
            &lstr(&self.resources.langbase, 247, "Next"),
            &lstr(&self.resources.langbase, 248, "End"),
        ));
        els
    }

    fn draw_list(&self, els: &mut Vec<Element>, phase: usize) {
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

        let mut table = Table::new();
        table.push(Cell::left(title, 30, yy, FONT_DEFAULT));
        yy += 17;

        table.push(Cell::left(
            lstr(&self.resources.langbase, 166, "Name"),
            col[0],
            yy,
            FONT_NEW,
        ));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 167, "Pos"),
            col[1],
            yy,
            FONT_NEW,
        ));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 168, "Points"),
            col[2],
            yy,
            FONT_NEW,
        ));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 169, "Date"),
            col[3],
            yy,
            FONT_NEW,
        ));

        let records = self.store.records();
        for idx in start..start + entries {
            yy += 8;
            let Some(hi) = records.top(idx) else {
                continue;
            };
            self.push_hiscore_row(&mut table, hi, idx - start + 1, yy, col, sortby);
        }

        els.extend(table.into_elements());
    }

    fn push_hiscore_row(
        &self,
        table: &mut Table,
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
        table.push(Cell::right(ordinal_dot(place), 24, y, FONT_NEW));
        table.push(Cell::left(
            shorten_name(&hi.name, &self.resources.font, 110),
            col[0],
            y,
            name_color,
        ));
        table.push(Cell::right(ordinal_dot(hi.pos), col[1] + 14, y, name_color));
        let score = if sortby_points {
            format_decimal(hi.score)
        } else {
            format!("{:.0}", hi.score)
        };
        table.push(Cell::right(score, col[2] + 24, y, name_color));
        table.push(Cell::left(&hi.time, col[3], y, FONT_HELP));
    }

    fn draw_koth_records(&self, els: &mut Vec<Element>) {
        let col = [30, 55, 175, 290];
        let mut yy = 12;
        let mut table = Table::new();
        table.push(Cell::left(
            lstr(&self.resources.langbase, 160, "King of the Hill"),
            30,
            6,
            FONT_DEFAULT,
        ));

        let records = self.store.records();
        for idx in 1..=6 {
            yy += 18;
            table.push(Cell::left(
                format!(
                    "{}. {}",
                    idx,
                    lstr(&self.resources.langbase, 130 + idx, "Challenge")
                ),
                col[0],
                yy,
                FONT_NEW,
            ));
            yy += 10;

            let name = lstr(&self.resources.langbase, 161, "Nobody");
            let Some(hi) = records.top(idx + 35) else {
                table.push(Cell::left(name, col[1], yy, FONT_HELP));
                continue;
            };
            if hi.score > 0.0 {
                table.push(Cell::left(&hi.time, col[2], yy, FONT_HELP));
                table.push(Cell::left(
                    format!("{:.0} X", hi.score),
                    col[3],
                    yy,
                    FONT_DEFAULT,
                ));
                table.push(Cell::left(&hi.name, col[1], yy, FONT_DEFAULT));
            } else {
                table.push(Cell::left(name, col[1], yy, FONT_HELP));
            }
        }

        els.extend(table.into_elements());
    }
}

impl View<RouteTarget> for HallOfFameView {
    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
        }
    }
}

impl Screen<RouteTarget> for HallOfFameView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
            return;
        };
        if let Some(route) = page_nav::handle_paged_event(event, &mut self.page, HALL_PAGES) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        paint_elements(cx, &self.legacy_elements());
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

    fn legacy_elements(&self) -> Vec<Element> {
        let pages = self.pages();
        let mut els = new_screen(1);
        self.draw_hill_records(&mut els);
        els.extend(page_hints(
            self.page,
            pages,
            &lstr(&self.resources.langbase, 246, "Back"),
            &lstr(&self.resources.langbase, 247, "Next"),
            &lstr(&self.resources.langbase, 248, "End"),
        ));
        els
    }

    fn draw_hill_records(&self, els: &mut Vec<Element>) {
        let col = [3, 71, 183, 200, 216];
        let phase = self.page;
        let start = phase * PAGE_SIZE;
        let loop_count = (self.resources.hills.len().saturating_sub(start)).min(PAGE_SIZE);
        let title = if phase == 0 {
            lstr(&self.resources.langbase, 170, "Hill Records")
        } else {
            lstr(&self.resources.langbase, 156, "Extra Hill Records")
        };
        let mut table = Table::new();
        table.push(Cell::left(title, 30, 6, FONT_DEFAULT));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 106, "Hill"),
            col[0],
            23,
            FONT_DEFAULT,
        ));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 171, "Who"),
            col[1],
            23,
            FONT_DEFAULT,
        ));
        table.push(Cell::right(
            lstr(&self.resources.langbase, 172, "Length"),
            col[2],
            23,
            FONT_DEFAULT,
        ));
        table.push(Cell::left("(K)", col[3], 23, FONT_DEFAULT));
        table.push(Cell::left(
            lstr(&self.resources.langbase, 169, "Date"),
            col[4],
            23,
            FONT_DEFAULT,
        ));

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

            table.push(Cell::left(
                shorten_name(&hill.name, &self.resources.font, 64),
                col[0],
                y,
                FONT_NEW,
            ));
            let record_color = if is_computer_name(&record.name) {
                FONT_GREET
            } else {
                FONT_DEFAULT
            };
            table.push(Cell::left(
                shorten_name(&record.name, &self.resources.font, 80),
                col[1],
                y,
                record_color,
            ));
            let length_color = if is_computer_name(&record.name) {
                FONT_GREET
            } else {
                FONT_NEW
            };
            table.push(Cell::right(
                format_decimal(record.len),
                col[2],
                y,
                length_color,
            ));
            table.push(Cell::right(
                format!("({})", hill.kr),
                col[3] + 11,
                y,
                length_color,
            ));
            table.push(Cell::left(record.time, col[4], y, FONT_HELP));
        }

        if phase == 0 {
            let total: f64 = (0..self.resources.hills.len().min(PAGE_SIZE))
                .filter_map(|idx| self.resources.hills.hill(idx).map(|hill| hill.kr as f64))
                .sum();
            if total > 0.0 {
                table.push(Cell::left("A.H.I.", 130, 192, FONT_HELP));
                table.push(Cell::right(format_ahi(ahi_sum, total), 197, 192, FONT_HELP));
            }
        }

        els.extend(table.into_elements());
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

impl View<RouteTarget> for HillRecordsView {
    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
        }
    }
}

impl Screen<RouteTarget> for HillRecordsView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
            return;
        };
        let pages = self.pages();
        if let Some(route) = page_nav::handle_paged_event(event, &mut self.page, pages) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        paint_elements(cx, &self.legacy_elements());
    }
}
