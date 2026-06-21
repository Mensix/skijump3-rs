use crate::components::modal::alert_prompt;
use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;

use engine::oxide::input::Key;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};
use serde::Deserialize;

#[derive(Debug)]
struct CustomHillListEntry {
    filename: String,
    hillname: String,
}

#[derive(Deserialize)]
struct CustomHillCatalogToml {
    hills: Vec<CustomHillToml>,
}

#[derive(Deserialize)]
struct CustomHillToml {
    name: String,
}

pub struct HillMakerView {
    resources: ResourcesRef,
    menu: PixelMenu,
    custom_hills: Vec<CustomHillListEntry>,
    page_start: usize,
    mode: HillMakerMode,
}

#[derive(Debug, Clone)]
enum HillMakerMode {
    Browse,
    ConfirmDelete { filename: String },
}

const PAGE_SIZE: usize = 18;

impl HillMakerView {
    pub fn new(resources: ResourcesRef) -> Self {
        let custom_hills = Self::load_custom_hills(&resources);
        let page_start = 0;
        let menu = Self::make_menu(custom_hills.len(), page_start);
        Self {
            resources,
            menu,
            custom_hills,
            page_start,
            mode: HillMakerMode::Browse,
        }
    }

    fn make_menu(total_hills: usize, page_start: usize) -> PixelMenu {
        let visible = Self::visible_count(total_hills, page_start);
        let mut count = visible + 1; // add new
        if page_start + visible < total_hills {
            count += 1;
        }
        if page_start > 0 {
            count += 1;
        }
        let items = (1..=count).map(|n| MenuItem::new(n as u8, "")).collect();
        let exit_gap = 14;
        PixelMenu::new(99, 14, 221, 8, items, FONT_BODY, FONT_BODY)
            .trailing("", exit_gap)
            .with_labels(false)
    }

    fn visible_count(total_hills: usize, page_start: usize) -> usize {
        total_hills.saturating_sub(page_start).min(PAGE_SIZE)
    }

    fn rebuild_menu(&mut self) {
        self.menu = Self::make_menu(self.custom_hills.len(), self.page_start);
    }

    fn page_count(&self) -> usize {
        self.custom_hills.len().max(1).div_ceil(PAGE_SIZE)
    }

    fn page_number(&self) -> usize {
        self.page_start / PAGE_SIZE + 1
    }

    fn item_roles(&self) -> (usize, usize, Option<usize>, Option<usize>) {
        let visible = Self::visible_count(self.custom_hills.len(), self.page_start);
        let add = visible + 1;
        let mut next = None;
        let mut prev = None;
        let mut item = add;
        if self.page_start + visible < self.custom_hills.len() {
            item += 1;
            next = Some(item);
        }
        if self.page_start > 0 {
            item += 1;
            prev = Some(item);
        }
        (visible, add, next, prev)
    }

    fn load_custom_hills(resources: &ResourcesRef) -> Vec<CustomHillListEntry> {
        let mut names = resources
            .files
            .list_save_subdir_by_ext("custom_hills", "toml");
        names.sort();
        names
            .into_iter()
            .filter_map(|name| {
                let path = format!("custom_hills/{name}");
                let data = resources.files.read(&path);
                let text = std::str::from_utf8(&data).ok()?;
                let catalog = toml::from_str::<CustomHillCatalogToml>(text).ok()?;
                let hillname = catalog.hills.first()?.name.clone();
                let filename = name.strip_suffix(".toml").unwrap_or(&name).to_string();
                Some(CustomHillListEntry { filename, hillname })
            })
            .collect()
    }
}

impl GameScreen for HillMakerView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let HillMakerMode::ConfirmDelete { filename } = self.mode.clone() {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                if matches!(event, UiEvent::Text('Y' | 'y')) {
                    let path = format!("custom_hills/{filename}.toml");
                    let _ = self.resources.files.delete_save(&path);
                    self.custom_hills = Self::load_custom_hills(&self.resources);
                    if self.page_start >= self.custom_hills.len() {
                        self.page_start = self.page_start.saturating_sub(PAGE_SIZE);
                    }
                    self.rebuild_menu();
                }
                self.mode = HillMakerMode::Browse;
                nav.consume();
            }
            return;
        }

        if matches!(event, UiEvent::KeyDown(Key::Escape)) {
            nav.back();
            return;
        }
        if matches!(event, UiEvent::KeyDown(Key::Delete | Key::Backspace)) {
            let selected = self.menu.selected();
            let (visible, _, _, _) = self.item_roles();
            if selected < visible {
                let filename = self.custom_hills[self.page_start + selected]
                    .filename
                    .clone();
                self.mode = HillMakerMode::ConfirmDelete { filename };
                nav.consume();
            }
            return;
        }
        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(n) if n > 0 => {
                let (visible, add, next, prev) = self.item_roles();
                if n <= visible {
                    let filename = self.custom_hills[self.page_start + n - 1].filename.clone();
                    cx.state.nav_edit_hill = Some(filename);
                    nav.navigate(RouteTarget::EditHill);
                } else if n == add {
                    cx.state.nav_edit_hill = None;
                    nav.navigate(RouteTarget::EditHill);
                } else if Some(n) == next {
                    self.page_start += PAGE_SIZE;
                    self.rebuild_menu();
                    nav.consume();
                } else if Some(n) == prev {
                    self.page_start = self.page_start.saturating_sub(PAGE_SIZE);
                    self.rebuild_menu();
                    nav.consume();
                }
            }
            Some(0) => nav.back(),
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        let lb = &*self.resources.langbase;

        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        paint.text((5, 5), FONT_GOLD, lb.lstr_or( 270, "SJ3 Hill Maker"));

        paint.text((5, 21), FONT_GRAY, lb.lstr_or( 271, "(use arrows, DEL,"));
        paint.text((5, 29), FONT_GRAY, lb.lstr_or( 272, " ENTER or ESC)"));

        let col1 = 100i32;
        let col2 = 160i32;

        paint.text((col1, 5), FONT_TEAL, lb.lstr_or( 273, "Filename"));
        paint.text((col2, 5), FONT_TEAL, lb.lstr_or( 274, "Hillname"));

        paint.text(
            (5, 45),
            FONT_TEAL,
            format!(
                "{} {} {} {}",
                lb.lstr_or( 157, "Page"),
                self.page_number(),
                lb.lstr_or( 8, "of"),
                self.page_count()
            ),
        );

        let (visible, _, next, prev) = self.item_roles();
        for (i, hill) in self
            .custom_hills
            .iter()
            .skip(self.page_start)
            .take(visible)
            .enumerate()
        {
            let y = 13 + i as i32 * 8;
            paint.text((col1, y), FONT_BODY, &hill.filename);
            paint.text((col2, y), FONT_GOLD, &hill.hillname);
        }
        let mut row = visible;
        paint.text(
            (col1, 13 + row as i32 * 8),
            FONT_GOLD,
            lb.lstr_or( 275, "*Add New Hill*"),
        );
        row += 1;
        if next.is_some() {
            paint.text(
                (col1, 13 + row as i32 * 8),
                FONT_GRAY,
                lb.lstr_or( 158, "*Next Page*"),
            );
            row += 1;
        }
        if prev.is_some() {
            paint.text(
                (col1, 13 + row as i32 * 8),
                FONT_GRAY,
                lb.lstr_or( 159, "*Previous Page*"),
            );
            row += 1;
        }
        self.menu.paint(paint);

        let exit_y = 13 + (row + 2) as i32 * 8;
        paint.text((col1, exit_y), FONT_BODY, lb.lstr_or( 276, "-Exit-"));
        if let HillMakerMode::ConfirmDelete { filename } = &self.mode {
            alert_prompt(
                paint,
                format!("DELETE {filename}.TOML?"),
                lb.lstr_or( 193, "ARE YOU SURE?"), true,
            );
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}
