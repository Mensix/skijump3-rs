use crate::components::modal::{confirmation_choice, ConfirmationChoice, Modal};
use crate::content::hills::custom_hill_details;
use crate::files::FileStore;
use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::text::layout::shorten_name;
use crate::ui::{EventCx, MenuRenderSnapshot, UiCanvas};

use crate::ui::{Key, MenuAction, MenuItem, PixelMenu, ScreenBackground, ScreenEventCx, UiEvent};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq)]
struct CustomHillListEntry {
    filename: String,
    hillname: String,
    kr: i64,
}

pub struct HillMakerView {
    resources: ResourcesRef,
    menu: PixelMenu,
    custom_hills: Vec<CustomHillListEntry>,
    custom_hills_signature: u64,
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
        let custom_hills = Self::load_custom_hills(&resources.files);
        let custom_hills_signature = Self::custom_hills_signature(&resources.files);
        let page_start = 0;
        let menu = Self::make_menu(custom_hills.len(), page_start);
        Self {
            resources,
            menu,
            custom_hills,
            custom_hills_signature,
            page_start,
            mode: HillMakerMode::Browse,
        }
    }

    fn menu_items(total_hills: usize, page_start: usize) -> Vec<MenuItem> {
        let visible = Self::visible_count(total_hills, page_start);
        let mut count = visible + 1;
        if page_start + visible < total_hills {
            count += 1;
        }
        if page_start > 0 {
            count += 1;
        }
        let mut items: Vec<MenuItem> = (0..count).map(|n| MenuItem::new(n as u8, "")).collect();
        items.push(MenuItem::new(count as u8, "").with_gap_before(6));
        items
    }

    fn make_menu(total_hills: usize, page_start: usize) -> PixelMenu {
        PixelMenu::new(
            99,
            14,
            221,
            8,
            Self::menu_items(total_hills, page_start),
            FONT_BODY,
            FONT_BODY,
        )
        .with_labels(false)
    }

    fn visible_count(total_hills: usize, page_start: usize) -> usize {
        total_hills.saturating_sub(page_start).min(PAGE_SIZE)
    }

    fn rebuild_menu(&mut self) {
        let items = Self::menu_items(self.custom_hills.len(), self.page_start);
        self.menu.set_items(items);
    }

    fn page_count(&self) -> usize {
        self.custom_hills.len().max(1).div_ceil(PAGE_SIZE)
    }

    fn page_number(&self) -> usize {
        self.page_start / PAGE_SIZE + 1
    }

    fn item_roles(&self) -> (usize, usize, Option<usize>, Option<usize>) {
        let visible = Self::visible_count(self.custom_hills.len(), self.page_start);
        let add = visible;
        let mut item = add + 1;
        let mut next = None;
        let mut prev = None;
        if self.page_start + visible < self.custom_hills.len() {
            next = Some(item);
            item += 1;
        }
        if self.page_start > 0 {
            prev = Some(item);
        }
        (visible, add, next, prev)
    }

    fn load_custom_hills(files: &FileStore) -> Vec<CustomHillListEntry> {
        let mut names = files.list_save_subdir_by_ext("custom_hills", "toml");
        names.sort();
        names
            .into_iter()
            .filter_map(|name| {
                let (hillname, kr) = custom_hill_details(files, &name)?;
                let filename = name.strip_suffix(".toml").unwrap_or(&name).to_string();
                Some(CustomHillListEntry {
                    filename,
                    hillname,
                    kr,
                })
            })
            .collect()
    }

    fn custom_hills_signature(files: &FileStore) -> u64 {
        let mut names = files.list_save_subdir_by_ext("custom_hills", "toml");
        names.sort();
        let mut hasher = DefaultHasher::new();
        for name in names {
            name.hash(&mut hasher);
            files
                .read_save(&format!("custom_hills/{name}"))
                .hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl GameScreen for HillMakerView {
    fn update(&mut self, _: &mut GameCx<'_>) {
        let signature = Self::custom_hills_signature(&self.resources.files);
        if signature == self.custom_hills_signature {
            return;
        }
        self.custom_hills_signature = signature;
        self.resources.refresh_hills();
        self.custom_hills = Self::load_custom_hills(&self.resources.files);
        while self.page_start >= self.custom_hills.len() && self.page_start > 0 {
            self.page_start = self.page_start.saturating_sub(PAGE_SIZE);
        }
        self.rebuild_menu();
    }

    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let HillMakerMode::ConfirmDelete { filename } = self.mode.clone() {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                if matches!(event, UiEvent::Text(c) if confirmation_choice(c, &self.resources.langbase) == Some(ConfirmationChoice::Yes))
                {
                    let path = format!("custom_hills/{filename}.toml");
                    self.resources.files.delete_save(&path);
                    self.resources.refresh_hills();
                    self.custom_hills = Self::load_custom_hills(&self.resources.files);
                    self.custom_hills_signature =
                        Self::custom_hills_signature(&self.resources.files);
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

        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            nav.back();
            return;
        }

        if matches!(event, UiEvent::KeyDown(Key::Delete)) {
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
        let mut ecx = EventCx::default();
        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(n)) if n == self.menu.item_count() - 1 => nav.back(),
            Some(MenuAction::Item(n)) => {
                let (visible, add, next, prev) = self.item_roles();
                if n < visible {
                    let filename = self.custom_hills[self.page_start + n].filename.clone();
                    nav.navigate(RouteTarget::EditHill(Some(filename)));
                } else if n == add {
                    nav.navigate(RouteTarget::EditHill(Some(format!(
                        "NEW{}",
                        self.custom_hills.len() + 1
                    ))));
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
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        let lang = &*self.resources.langbase;

        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        paint.text((5, 5), FONT_GOLD, lang.tr(270));

        paint.text((5, 21), FONT_GRAY, lang.tr(271));
        paint.text((5, 29), FONT_GRAY, lang.tr(272));

        let col1 = 100i32;
        let col2 = 160i32;

        paint.text((col1, 5), FONT_TEAL, lang.tr(273));
        paint.text((col2, 5), FONT_TEAL, lang.tr(274));

        paint.text(
            (5, 45),
            FONT_TEAL,
            &format!(
                "{} {} {} {}",
                lang.tr(157),
                self.page_number(),
                lang.tr(8),
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
            let hill_parameter = format!(" K{}", hill.kr);
            let name_width = 150 - self.resources.font.string_width(&hill_parameter) as i32;
            let hill_name = shorten_name(&hill.hillname, &self.resources.font, name_width);
            paint.text((col2, y), FONT_GOLD, &hill_name);
            let parameter_x = col2 + self.resources.font.string_width(&hill_name) as i32;
            paint.text((parameter_x, y), FONT_TEAL, &hill_parameter);
        }
        let mut row = visible;
        paint.text((col1, 13 + row as i32 * 8), FONT_GOLD, lang.tr(275));
        row += 1;
        if next.is_some() {
            paint.text((col1, 13 + row as i32 * 8), FONT_GRAY, lang.tr(158));
            row += 1;
        }
        if prev.is_some() {
            paint.text((col1, 13 + row as i32 * 8), FONT_GRAY, lang.tr(159));
            row += 1;
        }
        let snapshot = MenuRenderSnapshot::from_menu(&self.menu);
        paint.paint_pixel_menu(&snapshot);

        let exit_y = 5 + (row + 2) as i32 * 8;
        paint.text((col1, exit_y), FONT_BODY, lang.tr(276));
        if let HillMakerMode::ConfirmDelete { filename } = &self.mode {
            Modal::confirm(format!("DELETE {filename}.TOML?"), lang.tr(193)).paint(paint, lang);
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn malformed_custom_hill_is_not_listed() {
        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        fs::create_dir_all(saves.path().join("custom_hills")).unwrap();
        fs::write(
            saves.path().join("custom_hills/test.toml"),
            "[[hills]]\nname = \"missing required fields\"\n",
        )
        .unwrap();
        let files = FileStore::new(assets.path().into(), saves.path().into());

        let hills = HillMakerView::load_custom_hills(&files);

        assert!(hills.is_empty());
    }
}
