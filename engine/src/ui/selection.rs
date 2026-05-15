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
}
