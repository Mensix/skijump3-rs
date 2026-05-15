/// Wrapping selection state for up/down navigation.
#[derive(Debug, Clone)]
pub struct SelectionState {
    selected: usize,
    count: usize,
}

impl SelectionState {
    pub fn new(count: usize) -> Self {
        Self { selected: 0, count }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.selected = idx.min(self.count.saturating_sub(1));
    }

    pub fn up(&mut self) {
        self.selected = if self.selected == 0 {
            self.count.saturating_sub(1)
        } else {
            self.selected - 1
        };
    }

    pub fn down(&mut self) {
        self.selected = if self.selected + 1 >= self.count {
            0
        } else {
            self.selected + 1
        };
    }

    pub fn resize(&mut self, count: usize) {
        self.count = count;
        self.selected = self.selected.min(self.count.saturating_sub(1));
    }

    /// Handle a digit key press for Pascal‑style numbered menus.
    ///
    /// `'1'` → index 0, `'2'` → 1, …, `'9'` → 8.
    /// Returns `Some(index)` if the digit maps to a valid item, `None` otherwise.
    /// `'0'` is **not** handled here (callers treat it as EXIT).
    pub fn handle_digit(&mut self, c: char) -> Option<usize> {
        let d = c.to_digit(10)?;
        let d = d as usize;
        if d >= 1 && d <= self.count {
            let idx = d - 1;
            self.selected = idx;
            Some(idx)
        } else {
            None
        }
    }
}
