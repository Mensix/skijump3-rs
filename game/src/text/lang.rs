use std::cell::Cell;

/// Metadata about one available language.
#[derive(Debug, Clone)]
pub struct Language {
    pub id: String,
    pub name: String,
}

/// Central translation registry.
#[derive(Debug, Clone)]
pub struct LangBase {
    strings: Vec<Vec<String>>,
    info: Vec<Language>,
    selected: Cell<usize>,
    fallback_lang: usize,
}

impl LangBase {
    #[must_use]
    pub(crate) fn new(
        strings: Vec<Vec<String>>,
        info: Vec<Language>,
        fallback_lang: usize,
    ) -> Self {
        Self {
            strings,
            info,
            selected: Cell::new(0),
            fallback_lang,
        }
    }

    // --- Primary lookup API ---

    /// Look up `index` in the currently selected language.
    /// Falls back to the default language, then returns `"?"`.
    pub fn tr(&self, index: usize) -> &str {
        self.try_tr(index, self.selected.get())
            .or_else(|| self.try_tr(index, self.fallback_lang))
            .unwrap_or("?")
    }

    fn try_tr(&self, index: usize, lang_idx: usize) -> Option<&str> {
        if lang_idx < self.strings.len() && index < self.strings[lang_idx].len() {
            let s = &self.strings[lang_idx][index];
            if s != "?" { Some(s) } else { None }
        } else {
            None
        }
    }

    // --- Language metadata ---

    pub fn languages(&self) -> &[Language] {
        &self.info
    }

    pub fn language_count(&self) -> usize {
        self.info.len()
    }

    // --- Language selection ---

    /// Current language index.
    pub fn selected(&self) -> usize {
        self.selected.get()
    }

    /// Set the current language by index. Returns `false` if the index was out of range.
    pub fn select(&self, index: usize) -> bool {
        if index < self.info.len() {
            self.selected.set(index);
            true
        } else {
            false
        }
    }

    pub fn is_saved_language_valid(&self, saved_value: i32) -> bool {
        saved_value >= 0 && (saved_value as usize) < self.info.len()
    }

    /// Read a persisted language value and apply it.
    /// Returns `true` if the value was valid and applied.
    pub fn apply_saved_language(&self, saved_value: i32) -> bool {
        if self.is_saved_language_valid(saved_value) {
            self.selected.set(saved_value as usize);
            true
        } else {
            false
        }
    }

    /// Serialize the current selection to a persistable `i32`.
    pub fn saved_language(&self) -> i32 {
        self.selected.get() as i32
    }
}
