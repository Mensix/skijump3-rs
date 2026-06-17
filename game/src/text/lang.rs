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

    /// Concatenate a range of language strings [start..=end], Pascal lrstr() style.
    pub fn lrstr(&self, start: usize, end: usize) -> String {
        let lang = self.selected.get();
        let strings = self
            .all_strings
            .get(lang)
            .map_or(&[] as &[String], |v| v.as_slice());
        (start..=end)
            .filter_map(|i| strings.get(i).map(|s| s.as_str()))
            .collect()
    }
}
