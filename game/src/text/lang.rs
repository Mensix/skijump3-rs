use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct LangBase {
    all_strings: Vec<Vec<String>>,
    pub languages: Vec<String>,
    pub selected: Cell<usize>,
}

impl LangBase {
    #[must_use]
    pub(crate) fn new(all_strings: Vec<Vec<String>>, languages: Vec<String>) -> Self {
        Self {
            all_strings,
            languages,
            selected: Cell::new(0),
        }
    }

    #[must_use]
    pub fn lstr(&self, index: usize) -> &str {
        let lang = self.selected.get();
        if lang < self.all_strings.len() && index < self.all_strings[lang].len() {
            &self.all_strings[lang][index]
        } else {
            "?"
        }
    }

    #[must_use]
    pub fn lstr_or(&self, index: usize, fallback: &str) -> String {
        let v = self.lstr(index);
        if v == "?" { fallback.to_string() } else { v.to_string() }
    }
}
