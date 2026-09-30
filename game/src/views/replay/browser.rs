use crate::components::detail_panel::{paint_detail_panel, DetailPanel};
use crate::components::layout::MainLayout;
use crate::components::modal::{confirmation_choice, ConfirmationChoice, Modal};
use crate::components::page_nav::cycle_index;
use crate::data::hill::HillCatalog;
use crate::files::FileStore;
use crate::gfx::theme::{BG_DARK, FONT_GOLD, FONT_GRAY};
use crate::jump::replay::ReplayTrace;
use crate::route::{ReplayReturn, RouteTarget};
use crate::screen::{GameCx, GameScreen};
use crate::store::{Resources, ResourcesRef};
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenBackground, ScreenEventCx, UiEvent};
use crate::views::menu::paint_numbered_menu;
use std::path::Path;

#[derive(Debug, Clone)]
struct ReplayEntry {
    filename: String,
    source_filename: String,
    trace: Option<ReplayTrace>,
    invalid_reason: Option<String>,
}

pub struct ReplayBrowserView {
    resources: ResourcesRef,
    entries: Vec<ReplayEntry>,
    selected: usize,
    confirm_delete: bool,
}

impl ReplayBrowserView {
    pub fn new(resources: ResourcesRef) -> Self {
        let entries = load_replays(&resources.files, &resources.hills);
        Self {
            resources,
            entries,
            selected: 0,
            confirm_delete: false,
        }
    }

    fn selected_entry(&self) -> Option<&ReplayEntry> {
        selected_entry(&self.entries, self.selected)
    }

    fn move_next(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), 1);
    }

    fn move_prev(&mut self) {
        self.selected = cycle_index(self.selected, self.entries.len(), -1);
    }

    fn delete_selected(&mut self) -> bool {
        let Some(filename) = self
            .selected_entry()
            .map(|entry| entry.source_filename.clone())
        else {
            self.confirm_delete = false;
            return false;
        };
        self.resources.files.delete_save(&filename);
        self.entries = load_replays(&self.resources.files, &self.resources.hills);
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.confirm_delete = false;
        self.entries.is_empty()
    }
}

impl GameScreen for ReplayBrowserView {
    fn event(&mut self, _: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.confirm_delete {
            match event {
                UiEvent::KeyDown(Key::Escape | Key::F10) => self.confirm_delete = false,
                UiEvent::Text(c)
                    if confirmation_choice(c, &self.resources.langbase)
                        == Some(ConfirmationChoice::Yes) =>
                {
                    if self.delete_selected() {
                        nav.back();
                    }
                }
                UiEvent::Text(c)
                    if confirmation_choice(c, &self.resources.langbase)
                        == Some(ConfirmationChoice::No) =>
                {
                    self.confirm_delete = false;
                }
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(..) => {
                    self.confirm_delete = false
                }
                _ => {}
            }
            nav.consume();
            return;
        }

        match event {
            UiEvent::KeyDown(Key::Escape | Key::F10) => {
                nav.back();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Right | Key::Down) | UiEvent::Text(' ' | '+') => {
                self.move_next();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Left | Key::Up) | UiEvent::Text('-') => {
                self.move_prev();
                nav.consume();
            }
            UiEvent::KeyDown(Key::Enter) => {
                if let Some(trace) = self.selected_entry().and_then(ReplayEntry::playable_trace) {
                    nav.navigate(RouteTarget::ReplayPlayback {
                        trace: Box::new(trace),
                        return_to: ReplayReturn::Browser,
                    });
                }
            }
            UiEvent::KeyDown(Key::Delete) => {
                if self
                    .selected_entry()
                    .is_some_and(|entry| self.resources.files.exists_save(&entry.source_filename))
                {
                    self.confirm_delete = true;
                }
                nav.consume();
            }
            UiEvent::KeyDown(_)
            | UiEvent::Text(_)
            | UiEvent::TextWithModifiers(..)
            | UiEvent::Quit
            | UiEvent::Tick => {}
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        let lang = &cx.layout.langbase;
        paint.fill((11, 80, 100, 6), BG_DARK);
        paint.text((11, 80), FONT_GOLD, lang.tr(17));
        paint_replay_menu(paint, cx.layout);
        cx.layout.footer(paint);
        paint_replay_panel(paint, &self.resources, &self.entries, self.selected);
        if self.confirm_delete {
            let are_you_sure = lang.tr(193);
            if let Some(entry) = self.selected_entry() {
                paint_delete_confirm(paint, &entry.filename, are_you_sure, lang);
            }
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_delete_confirm(cx: &mut dyn UiCanvas, filename: &str, message: &str, lang: &LangBase) {
    Modal::confirm(format!("Delete {filename}.SJR?"), message).paint(cx, lang);
}

fn paint_replay_panel(
    cx: &mut dyn UiCanvas,
    resources: &Resources,
    entries: &[ReplayEntry],
    selected: usize,
) {
    let lang = &resources.langbase;
    let Some(entry) = entries.get(selected) else {
        paint_detail_panel(
            cx,
            DetailPanel {
                lang,
                title: &format!("{}:", lang.tr(25)),
                filename: "",
                filename_color: FONT_GOLD,
                fields: &[],
                extra_value: None,
                counter: None,
                nav_hint: lang.tr(146),
                empty_text: lang.tr(290),
                is_empty: true,
            },
        );
        return;
    };

    let Some(trace) = entry.trace.as_ref() else {
        paint_invalid_replay_panel(cx, lang, entry, selected, entries.len());
        return;
    };
    if !trace.meta.valid_checksum || entry.invalid_reason.is_some() {
        paint_invalid_replay_panel(cx, lang, entry, selected, entries.len());
        return;
    }

    let hill = resources.hills.hill(trace.meta.hill_idx).map_or_else(
        || "?".to_string(),
        |hill| format!("{} K{}", hill.name, hill.kr),
    );
    let field_pairs = vec![
        (lang.tr(291).to_string(), trace.meta.author.clone()),
        (lang.tr(292).to_string(), trace.meta.name.clone()),
        (lang.tr(294).to_string(), hill),
    ];
    let extra_saved = Some(trace.meta.saved_at.clone());

    paint_detail_panel(
        cx,
        DetailPanel {
            lang,
            title: &format!("{}:", lang.tr(25)),
            filename: &entry.filename,
            filename_color: FONT_GOLD,
            fields: &field_pairs,
            extra_value: extra_saved.as_deref(),
            counter: Some((selected + 1, entries.len())),
            nav_hint: lang.tr(146),
            empty_text: lang.tr(290),
            is_empty: false,
        },
    );
}

fn paint_replay_menu(cx: &mut dyn UiCanvas, layout: &MainLayout) {
    paint_numbered_menu(
        cx,
        &layout.langbase,
        [20, 21, 22, 23, 24, 25, 26],
        [0, 0, 0, 0, 0, 0, 12],
        None,
    );
}

fn load_replays(files: &FileStore, hills: &HillCatalog) -> Vec<ReplayEntry> {
    let names = files.list_replays();
    names
        .into_iter()
        .map(|filename| {
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&filename)
                .to_string();
            let intro = stem.eq_ignore_ascii_case("INTRO");
            let data = files.read_save_or_asset(&filename);
            let parsed = ReplayTrace::from_sjr_bytes(&data, intro);
            let (trace, invalid_reason) = match parsed {
                Ok(mut trace) if trace.meta.valid_checksum => {
                    let resolved = hills.replay_hill_index(
                        trace.meta.hill_idx,
                        &trace.meta.hill_filename,
                        trace.meta.hill_profile,
                    );
                    match resolved {
                        Some(hill_idx) => {
                            trace.meta.hill_idx = hill_idx;
                            (Some(trace), None)
                        }
                        None => (Some(trace), Some("Extra Hill Not Found.".to_string())),
                    }
                }
                Ok(trace) => (Some(trace), Some("Checksum mismatch.".to_string())),
                Err(error) => (None, Some(error.to_string())),
            };
            ReplayEntry {
                filename: stem,
                source_filename: filename,
                trace,
                invalid_reason,
            }
        })
        .collect()
}

fn selected_entry(entries: &[ReplayEntry], selected: usize) -> Option<&ReplayEntry> {
    entries.get(selected)
}

impl ReplayEntry {
    fn playable_trace(&self) -> Option<ReplayTrace> {
        self.trace
            .as_ref()
            .filter(|trace| trace.meta.valid_checksum && self.invalid_reason.is_none())
            .cloned()
    }
}

fn paint_invalid_replay_panel(
    cx: &mut dyn UiCanvas,
    lang: &LangBase,
    entry: &ReplayEntry,
    selected: usize,
    total: usize,
) {
    paint_detail_panel(
        cx,
        DetailPanel {
            lang,
            title: &format!("{}:", lang.tr(25)),
            filename: &entry.filename,
            filename_color: FONT_GRAY,
            fields: &[
                (lang.tr(291).to_string(), "Unknown".to_string()),
                (lang.tr(292).to_string(), "Not a valid replay.".to_string()),
                (
                    lang.tr(294).to_string(),
                    entry
                        .invalid_reason
                        .clone()
                        .unwrap_or_else(|| "Not a valid replay.".to_string()),
                ),
            ],
            extra_value: Some("-"),
            counter: Some((selected + 1, total)),
            nav_hint: lang.tr(146),
            empty_text: lang.tr(290),
            is_empty: false,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn empty_browser_selection_and_navigation_are_safe() {
        assert!(selected_entry(&[], 0).is_none());
        assert_eq!(cycle_index(0, 0, 1), 0);
        assert_eq!(cycle_index(0, 0, -1), 0);
    }

    #[test]
    fn malformed_replay_stays_listed_with_reason_and_cannot_play() {
        let saves = tempdir().expect("save dir");
        std::fs::write(saves.path().join("BROKEN.SJR"), b"broken").expect("write replay");
        let files = FileStore::new(PathBuf::from("/nonexistent"), saves.path().to_path_buf());

        let entries = load_replays(&files, &HillCatalog::default());
        let broken = entries
            .iter()
            .find(|entry| entry.filename == "BROKEN")
            .expect("invalid replay remains listed");
        assert!(broken.invalid_reason.is_some());
        assert!(broken.playable_trace().is_none());
    }

    #[test]
    fn asset_only_intro_is_listed() {
        let files = FileStore::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );

        let entries = load_replays(&files, &HillCatalog::default());

        assert!(entries.iter().any(|entry| entry.filename == "INTRO"));
    }

    #[test]
    fn replay_list_has_no_artificial_limit() {
        let saves = tempdir().expect("save dir");
        for index in 0..101 {
            std::fs::write(saves.path().join(format!("REPLAY{index}.SJR")), b"broken")
                .expect("write replay");
        }
        let files = FileStore::new(PathBuf::from("/nonexistent"), saves.path().to_path_buf());

        let entries = load_replays(&files, &HillCatalog::default());

        assert_eq!(entries.len(), 101);
    }

    #[test]
    fn checksum_mismatch_cannot_play() {
        let files = FileStore::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let mut trace =
            ReplayTrace::from_sjr_bytes(&files.read("INTRO.SJR"), true).expect("intro replay");
        trace.meta.valid_checksum = false;
        let entry = ReplayEntry {
            filename: "INTRO".to_string(),
            source_filename: "INTRO.SJR".to_string(),
            trace: Some(trace),
            invalid_reason: Some("Checksum mismatch.".to_string()),
        };
        assert!(entry.playable_trace().is_none());
        assert_eq!(entry.invalid_reason.as_deref(), Some("Checksum mismatch."));
    }
}
