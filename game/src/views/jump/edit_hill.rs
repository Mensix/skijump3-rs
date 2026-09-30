use crate::components::modal::{confirmation_choice, ConfirmationChoice, Modal};
use crate::data::custom_hill::{CustomHill, CustomHillCatalog};
use crate::data::hill::{generated_hill_image_path, metadata_checksum, HillInfo};
use crate::data::hill_profile::profile_checksum;
use crate::files::FileStore;
use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::ui::{
    EventCx, Key, MenuAction, MenuItem, PixelMenu, ScreenBackground, ScreenEventCx, TextInput,
    TextInputMessage, TextInputParams, UiEvent, Widget,
};
use crate::ui::{MenuRenderSnapshot, TextInputRenderSnapshot, UiCanvas};

#[derive(Debug)]
enum EditMode {
    Viewing,
    Editing {
        field: usize,
        input: TextInput,
    },
    Alert {
        message: String,
        subtitle: String,
        old_value: String,
        field: usize,
    },
    ConfirmOverwrite {
        filename: String,
    },
    SaveResult(i32),
}

pub struct EditHillView {
    resources: ResourcesRef,
    menu: PixelMenu,
    mode: EditMode,
    values: [String; 12],
    initial_values: [String; 12],
}

impl EditHillView {
    pub fn new(resources: ResourcesRef, initial_filename: Option<String>) -> Self {
        let mut items: Vec<MenuItem> = (0..12).map(|n| MenuItem::new(n, "")).collect();
        items.push(MenuItem::new(12, "").with_gap_before(14));
        let menu = PixelMenu::new(10, 8, 110, 13, items, FONT_BODY, FONT_BODY).with_labels(false);
        let default_values = [
            "Default".into(),
            "120".into(),
            "1".into(),
            "0".into(),
            "100".into(),
            "0".into(),
            "100".into(),
            "100".into(),
            "6".into(),
            "Unknown".into(),
            initial_filename.clone().unwrap_or_else(|| "NEW1".into()),
            "".into(),
        ];
        let values = initial_filename
            .and_then(|filename| Self::load_values(&resources.files, &filename))
            .unwrap_or(default_values);
        let initial_values = values.clone();
        Self {
            resources,
            menu,
            mode: EditMode::Viewing,
            values,
            initial_values,
        }
    }

    fn load_values(files: &FileStore, filename: &str) -> Option<[String; 12]> {
        let path = format!("custom_hills/{filename}.toml");
        let data = if files.exists_save(&path) {
            files.read_save(&path)
        } else {
            files.read(&path)
        };
        let text = std::str::from_utf8(&data).ok()?;
        let catalog = toml::from_str::<CustomHillCatalog>(text).ok()?;
        let hill = catalog.hills.first()?;
        Some([
            hill.name.clone(),
            hill.kr.to_string(),
            hill.front_index.clone(),
            hill.back_index.clone(),
            hill.back_brightness.to_string(),
            (if hill.back_mirror { 1 } else { 0 }).to_string(),
            (hill.vx_final - 40).to_string(),
            hill.pk_hundred.to_string(),
            (hill.pl_save_ten_thousand - 3204).to_string(),
            hill.author.clone(),
            filename.to_string(),
            String::new(),
        ])
    }

    fn has_changes(&self) -> bool {
        self.values != self.initial_values
    }

    fn saved_catalog_exists(files: &FileStore, filename: &str) -> bool {
        files.exists_save(&format!("custom_hills/{filename}.toml"))
    }

    fn visual_exists(files: &FileStore, index: &str, prefix: &str) -> bool {
        !files
            .read(&generated_hill_image_path(index, prefix))
            .is_empty()
    }

    fn build_custom_hill_toml(&self) -> Option<CustomHillCatalog> {
        let filename = &self.values[10];
        let profile_checksum = profile_checksum(&self.resources.files, &self.values[2])?;
        let mut hill = CustomHill {
            id: filename.clone(),
            name: self.values[0].clone(),
            terrain_index: self.values[2].clone(),
            kr: self.values[1].parse().unwrap_or(120),
            front_index: self.values[2].clone(),
            back_index: self.values[3].clone(),
            back_brightness: self.values[4].parse().unwrap_or(100),
            back_mirror: self.values[5].parse::<i64>().unwrap_or(0) != 0,
            vx_final: self.values[6].parse::<i64>().unwrap_or(100) + 40,
            pk_hundred: self.values[7].parse().unwrap_or(100),
            pl_save_ten_thousand: self.values[8].parse::<i64>().unwrap_or(6) + 3204,
            author: self.values[9].clone(),
            checksum: 0,
            profile_checksum,
        };
        hill.checksum = metadata_checksum(&HillInfo {
            name: hill.name.clone(),
            kr: hill.kr,
            front_index: hill.front_index.clone(),
            back_index: hill.back_index.clone(),
            back_brightness: hill.back_brightness,
            back_mirror: i64::from(hill.back_mirror),
            vx_final: hill.vx_final,
            pk_hundred: hill.pk_hundred,
            pl_save_ten_thousand: hill.pl_save_ten_thousand,
            author: hill.author.clone(),
            profile_checksum,
            ..HillInfo::default()
        });
        Some(CustomHillCatalog {
            id: filename.clone(),
            name: format!("{} custom hill", filename),
            checksum_version: 1,
            hills: vec![hill],
        })
    }

    fn save(&self) -> bool {
        let Some(toml) = self.build_custom_hill_toml() else {
            return false;
        };
        let Ok(data) = toml::to_string(&toml) else {
            return false;
        };
        let filename = &self.values[10];
        let path = format!("custom_hills/{filename}.toml");
        if !self.resources.files.write(&path, data.as_bytes()) {
            return false;
        }
        self.resources.refresh_hills();
        true
    }

    fn start_edit(&mut self, field: usize) {
        let yy = 10 + (field * 13) as i32;
        let value = self.values[field].clone();
        let (max_width, max_chars) = match field {
            0 | 9 => (150, usize::MAX),
            2 | 3 => (20, 3),
            1 | 4 | 5 | 6 | 7 | 8 => (30, 10),
            10 => (62, 8),
            _ => (150, usize::MAX),
        };
        let input = TextInput::new(TextInputParams {
            x: 120,
            y: yy,
            max_width,
            initial: value,
            max_chars,
            bg: BLACK,
            fg: FONT_GOLD,
            cursor: FONT_BODY,
            font: self.resources.font.clone(),
        });
        self.mode = EditMode::Editing { field, input };
    }

    fn commit_edit(&mut self, mut value: String) {
        let field = match &self.mode {
            EditMode::Editing { field, .. } => *field,
            _ => return,
        };
        match field {
            1 => self.validate_or_keep(field, &mut value, 40, 300),
            2 | 3 => {
                if value.is_empty() || !value.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return;
                }
                let prefix = if field == 2 { "front" } else { "back" };
                if !Self::visual_exists(&self.resources.files, &value, prefix) {
                    let label = if field == 2 { "FRONT" } else { "BACK" };
                    let old = self.values[field].clone();
                    self.mode = EditMode::Alert {
                        field,
                        old_value: old,
                        message: format!("INVALID {label} INDEX VALUE."),
                        subtitle: format!("FILE {label}{value}.PNG DOESN'T EXIST."),
                    };
                    return;
                }
            }
            4 => self.validate_or_keep(field, &mut value, 0, 255),
            5 => self.validate_or_keep(field, &mut value, 0, 1),
            6 => self.validate_or_keep(field, &mut value, 60, 145),
            7 => self.validate_or_keep(field, &mut value, 50, 150),
            8 => self.validate_or_keep(field, &mut value, 0, 30),
            10 => value.truncate(8),
            _ => {}
        }
        self.values[field] = value;
        self.mode = EditMode::Viewing;
    }

    fn validate_or_keep(&self, field: usize, value: &mut String, low: i32, high: i32) {
        if !Self::validate_int(value, low, high) {
            *value = self.values[field].clone();
        }
    }

    fn validate_int(value: &mut String, low: i32, high: i32) -> bool {
        if let Ok(n) = value.trim().parse::<i32>() {
            *value = n.clamp(low, high).to_string();
            true
        } else {
            false
        }
    }

    fn cancel_edit(&mut self) {
        self.mode = EditMode::Viewing;
    }

    fn is_numeric_field(field: usize) -> bool {
        matches!(field, 1 | 4 | 5 | 6 | 7 | 8)
    }

    fn is_valid_filename(value: &str) -> bool {
        !value.is_empty() && value.len() <= 8 && value.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

impl GameScreen for EditHillView {
    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match &mut self.mode {
            EditMode::ConfirmOverwrite { .. } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    if matches!(event, UiEvent::Text(c) if confirmation_choice(c, &self.resources.langbase) == Some(ConfirmationChoice::Yes))
                    {
                        if self.save() {
                            self.mode = EditMode::SaveResult(0);
                        } else {
                            self.mode = EditMode::SaveResult(1);
                        }
                    } else {
                        self.mode = EditMode::Viewing;
                        self.menu.set_selected(10);
                    }
                }
                nav.consume();
                return;
            }
            EditMode::Alert {
                field, old_value, ..
            } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.values[*field] = old_value.clone();
                    self.mode = EditMode::Viewing;
                }
                nav.consume();
                return;
            }
            EditMode::SaveResult(result) => {
                let result = *result;
                let modal = Modal::result(3, result);
                if modal.event(event, &self.resources.langbase).is_some() {
                    if result == 0 {
                        nav.back();
                    } else {
                        self.mode = EditMode::Viewing;
                    }
                }
                nav.consume();
                return;
            }
            EditMode::Editing { field, input } => {
                let event = if Self::is_numeric_field(*field) {
                    match event {
                        UiEvent::Text(c) if !c.is_ascii_digit() => return,
                        _ => event,
                    }
                } else {
                    event
                };
                let mut ecx = EventCx::default();
                match input.event(&mut ecx, event) {
                    Some(TextInputMessage::Commit(value)) => {
                        self.commit_edit(value);
                    }
                    Some(TextInputMessage::Cancel) => {
                        self.cancel_edit();
                    }
                    None => {}
                }
                if ecx.is_consumed() {
                    nav.consume();
                }
                return;
            }
            EditMode::Viewing => {}
        }

        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            nav.back();
            return;
        }

        let mut ecx = EventCx::default();
        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(12)) => {
                if !self.has_changes() {
                    nav.back();
                } else {
                    let filename = &self.values[10];
                    if !Self::is_valid_filename(filename) {
                        self.mode = EditMode::Alert {
                            field: 10,
                            old_value: filename.clone(),
                            message: self.resources.langbase.tr(353).to_string(),
                            subtitle: self.resources.langbase.tr(15).to_string(),
                        };
                        return;
                    }
                    if Self::saved_catalog_exists(&self.resources.files, filename) {
                        self.mode = EditMode::ConfirmOverwrite {
                            filename: filename.clone(),
                        };
                    } else if self.save() {
                        self.mode = EditMode::SaveResult(0);
                    } else {
                        self.mode = EditMode::SaveResult(1);
                    }
                }
            }
            Some(MenuAction::Item(11)) => {
                self.values = self.initial_values.clone();
                nav.back();
            }
            Some(MenuAction::Item(n @ 0..=10)) => {
                if matches!(event, UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ')) {
                    self.start_edit(n);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let xx = 15i32;
        let xx2 = 120i32;

        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        let labels = [
            "HILL.NAME",
            "K.POINT",
            "FRONT INDEX",
            "BACK INDEX",
            "BACK BRIGHT",
            "MIRROR BACK",
            "VX.FINAL",
            "ADJUST.K",
            "AIR.P.PLUS",
            "AUTHOR",
            "FILENAME",
            "EXIT, DON'T SAVE",
        ];

        let descriptions = [
            "",
            "CRITICAL POINT (40-300\u{00b5})",
            "FRONT*.PCX (000-ZZZ)",
            "BACK*.PCX (000-ZZZ)",
            "PERCENTS OF ORIGINAL (0-255)",
            "0-NO, 1-MIRROR (0-1)",
            "TAKE-OFF SPEED (60-145 KM/H)",
            "MOVE K-POINT ON HILL (50-150)",
            "+ 990 MBAR (0-30)",
            "",
            "     8 CHARS MAX",
            "",
        ];

        for temp in 0..12 {
            let yy = 10 + (temp * 13) as i32;
            let label = if temp < 11 {
                format!("{}. {}:", temp + 1, labels[temp])
            } else {
                format!("{}. {}", temp + 1, labels[temp])
            };

            paint.text((xx, yy), FONT_BODY, &label);

            if !descriptions[temp].is_empty() {
                paint.text((xx2 + 40, yy), FONT_GRAY, descriptions[temp]);
            }

            if !self.values[temp].is_empty() {
                paint.text((xx2, yy), FONT_GOLD, &self.values[temp]);
            }
        }

        paint.text((xx, 179), FONT_GOLD, &format!("0. {}", lang.tr(296)));

        if matches!(self.mode, EditMode::Viewing | EditMode::Editing { .. }) {
            let snapshot = MenuRenderSnapshot::from_menu(&self.menu);
            paint.paint_pixel_menu(&snapshot);
        }

        match &self.mode {
            EditMode::Editing { input, .. } => {
                let snapshot = TextInputRenderSnapshot::from_input(input);
                paint.paint_text_input(&snapshot);
            }
            EditMode::Alert {
                ref message,
                ref subtitle,
                ..
            } => {
                let prompt = lang.tr(15);
                Modal::alert(message, format!("{subtitle}  {prompt}")).paint(paint, lang);
            }
            EditMode::ConfirmOverwrite { filename } => {
                let prompt = lang.tr(346);
                Modal::confirm(format!("FILE {filename}.TOML ALREADY EXISTS."), prompt)
                    .paint(paint, lang);
            }
            EditMode::SaveResult(result) => {
                Modal::result(3, *result).paint(paint, lang);
            }
            EditMode::Viewing => {}
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentStore;
    use crate::store::Resources;
    use crate::ui::Font;
    use std::fs;
    use std::rc::Rc;

    fn catalog(name: &str) -> String {
        format!(
            r#"id = "test"
 name = "test catalog"
 checksum_version = 1

[[hills]]
id = "test"
name = "{name}"
terrain_index = "1"
kr = 120
front_index = "1"
back_index = "0"
back_brightness = 100
back_mirror = false
vx_final = 140
pk_hundred = 100
pl_save_ten_thousand = 3210
author = "test"
checksum = 0
profile_checksum = 0
"#
        )
    }

    #[test]
    fn editor_loads_saved_catalog_before_asset_catalog() {
        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        fs::create_dir_all(assets.path().join("custom_hills")).unwrap();
        fs::create_dir_all(saves.path().join("custom_hills")).unwrap();
        fs::write(
            assets.path().join("custom_hills/test.toml"),
            catalog("asset hill"),
        )
        .unwrap();
        fs::write(
            saves.path().join("custom_hills/test.toml"),
            catalog("saved hill"),
        )
        .unwrap();
        let files = FileStore::new(assets.path().into(), saves.path().into());

        let values = EditHillView::load_values(&files, "test").unwrap();

        assert_eq!(values[0], "saved hill");
    }

    #[test]
    fn editor_falls_back_to_bundled_catalog() {
        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        fs::create_dir_all(assets.path().join("custom_hills")).unwrap();
        fs::write(
            assets.path().join("custom_hills/test.toml"),
            catalog("asset hill"),
        )
        .unwrap();
        let files = FileStore::new(assets.path().into(), saves.path().into());

        let values = EditHillView::load_values(&files, "test").unwrap();

        assert_eq!(values[0], "asset hill");
    }

    #[test]
    fn overwrite_detection_ignores_asset_only_catalogs() {
        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        fs::create_dir_all(assets.path().join("custom_hills")).unwrap();
        fs::write(
            assets.path().join("custom_hills/test.toml"),
            catalog("asset hill"),
        )
        .unwrap();
        let files = FileStore::new(assets.path().into(), saves.path().into());

        assert!(!EditHillView::saved_catalog_exists(&files, "test"));

        fs::create_dir_all(saves.path().join("custom_hills")).unwrap();
        fs::write(
            saves.path().join("custom_hills/test.toml"),
            catalog("saved hill"),
        )
        .unwrap();
        assert!(EditHillView::saved_catalog_exists(&files, "test"));
    }

    #[test]
    fn visual_validation_uses_runtime_png_names() {
        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        let hill_dir = assets.path().join("hills/generated/HILLTEST");
        fs::create_dir_all(&hill_dir).unwrap();
        fs::write(hill_dir.join("front_visual.png"), b"obsolete").unwrap();
        let files = FileStore::new(assets.path().into(), saves.path().into());

        assert!(!EditHillView::visual_exists(&files, "TEST", "front"));

        fs::write(hill_dir.join("front.png"), b"runtime front").unwrap();
        fs::write(hill_dir.join("back.png"), b"runtime back").unwrap();
        assert!(EditHillView::visual_exists(&files, "TEST", "front"));
        assert!(EditHillView::visual_exists(&files, "TEST", "back"));
    }

    #[test]
    fn save_refreshes_shared_catalog_invalidates_terrain_and_writes_checksums() {
        let saves = tempfile::tempdir().unwrap();
        let files = Rc::new(FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            saves.path().to_path_buf(),
        ));
        let content = ContentStore::load(&files).expect("test assets load");
        let resources = Rc::new(Resources::new(
            Font::default(),
            Rc::new(content.langbase),
            content.namesets,
            content.hills,
            files.clone(),
        ));
        let original_count = resources.hills.len();
        let _ = resources.terrain(0);
        assert_eq!(resources.terrain_cache.borrow().len(), 1);
        let view = EditHillView::new(resources.clone(), None);

        view.save();

        assert_eq!(resources.hills.len(), original_count + 1);
        assert!(resources.terrain_cache.borrow().is_empty());
        let data = files.read_save("custom_hills/NEW1.toml");
        let saved: CustomHillCatalog = toml::from_str(std::str::from_utf8(&data).unwrap()).unwrap();
        assert_eq!(saved.checksum_version, 1);
        assert_ne!(saved.hills[0].checksum, 0);
        assert_ne!(saved.hills[0].profile_checksum, 0);
    }

    #[test]
    fn filename_and_numeric_defaults_match_registered_contract() {
        assert!(EditHillView::is_valid_filename("12345678"));
        assert!(!EditHillView::is_valid_filename("123456789"));
        assert!(!EditHillView::is_valid_filename("NEW-1"));

        let assets = tempfile::tempdir().unwrap();
        let saves = tempfile::tempdir().unwrap();
        fs::create_dir_all(assets.path().join("custom_hills")).unwrap();
        fs::write(
            assets.path().join("custom_hills/test.toml"),
            catalog("hill"),
        )
        .unwrap();
        let values = EditHillView::load_values(
            &FileStore::new(assets.path().into(), saves.path().into()),
            "test",
        )
        .unwrap();
        assert_eq!(&values[6], "100");
        assert_eq!(&values[8], "6");
    }

    #[test]
    fn name_and_author_editing_keep_pascal_width_without_130_char_limit() {
        let saves = tempfile::tempdir().unwrap();
        let files = Rc::new(FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            saves.path().to_path_buf(),
        ));
        let content = ContentStore::load(&files).unwrap();
        let resources = Rc::new(Resources::new(
            Font::default(),
            Rc::new(content.langbase),
            content.namesets,
            content.hills,
            files,
        ));
        let mut view = EditHillView::new(resources, None);

        for field in [0, 9] {
            view.start_edit(field);
            let EditMode::Editing { input, .. } = &view.mode else {
                panic!("field should enter editing mode");
            };
            assert_eq!(input.max_width(), 150);
            assert!(format!("{input:?}").contains(&format!("max_chars: {}", usize::MAX)));
        }
    }

    #[test]
    fn save_reports_write_failure() {
        let saves = tempfile::tempdir().unwrap();
        fs::write(saves.path().join("custom_hills"), b"not a directory").unwrap();
        let files = Rc::new(FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            saves.path().to_path_buf(),
        ));
        let content = ContentStore::load(&files).unwrap();
        let resources = Rc::new(Resources::new(
            Font::default(),
            Rc::new(content.langbase),
            content.namesets,
            content.hills,
            files,
        ));
        assert!(!EditHillView::new(resources, None).save());
    }
}
