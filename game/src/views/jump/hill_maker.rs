use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_TEAL, FONT_GRAY};
use crate::route::RouteTarget;
use crate::store::ResourcesRef;
use crate::text::layout::lstr;
use engine::oxide::input::Key;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};
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
}

impl HillMakerView {
    pub fn new(resources: ResourcesRef) -> Self {
        let custom_hills = Self::load_custom_hills(&resources);
        let items = vec![MenuItem::new(1, "")];
        let menu = PixelMenu::new(99, 14, 221, 8, items, FONT_BODY, FONT_BODY)
            .trailing("", 6 + custom_hills.len() as i32 * 8)
            .with_labels(false);
        Self {
            resources,
            menu,
            custom_hills,
        }
    }

    fn load_custom_hills(resources: &ResourcesRef) -> Vec<CustomHillListEntry> {
        let Ok(mut names) = resources
            .files
            .list_save_subdir_by_ext("custom_hills", "toml")
        else {
            return Vec::new();
        };
        names.sort();
        names
            .into_iter()
            .filter_map(|name| {
                let path = format!("custom_hills/{name}");
                let data = resources.files.read(&path).ok()?;
                let text = std::str::from_utf8(&data).ok()?;
                let catalog = toml::from_str::<CustomHillCatalogToml>(text).ok()?;
                let hillname = catalog.hills.first()?.name.clone();
                let filename = name.strip_suffix(".toml").unwrap_or(&name).to_string();
                Some(CustomHillListEntry { filename, hillname })
            })
            .collect()
    }
}

impl Screen<RouteTarget> for HillMakerView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::KeyDown(Key::Escape)) {
            cx.back();
            return;
        }
        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(1) => cx.navigate(RouteTarget::EditHill),
            Some(0) => cx.back(),
            _ => {}
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        let lb = &*self.resources.langbase;

        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        cx.text((5, 5), FONT_GOLD, lstr(lb, 270, "SJ3 Hill Maker"));

        cx.text((5, 21), FONT_GRAY, lstr(lb, 271, "(use arrows, DEL,"));
        cx.text((5, 29), FONT_GRAY, lstr(lb, 272, " ENTER or ESC)"));

        let col1 = 100i32;
        let col2 = 160i32;

        cx.text((col1, 5), FONT_TEAL, lstr(lb, 273, "Filename"));
        cx.text((col2, 5), FONT_TEAL, lstr(lb, 274, "Hillname"));

        cx.text(
            (5, 45),
            FONT_TEAL,
            format!("{} 1 {} 1", lstr(lb, 157, "Page"), lstr(lb, 8, "of")),
        );

        cx.text((col1, 13), FONT_GOLD, lstr(lb, 275, "*Add New Hill*"));
        for (i, hill) in self.custom_hills.iter().enumerate() {
            let y = 21 + i as i32 * 8;
            cx.text((col1, y), FONT_BODY, &hill.filename);
            cx.text((col2, y), FONT_GOLD, &hill.hillname);
        }
        self.menu.paint(cx);

        let exit_y = 29 + self.custom_hills.len() as i32 * 8;
        cx.text((col1, exit_y), FONT_BODY, lstr(lb, 276, "-Exit-"));
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}
