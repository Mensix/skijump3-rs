#[derive(Debug, Clone)]
pub struct TextEditState {
    buffer: String,
    cursor: usize,
    max_chars: usize,
}

impl TextEditState {
    pub fn new(initial: String, max_chars: usize) -> Self {
        let cursor = initial.chars().count();
        Self {
            buffer: initial,
            cursor,
            max_chars,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn char_count(&self) -> usize {
        self.buffer.chars().count()
    }

    pub fn cursor(&self) -> usize {
        self.cursor.min(self.char_count())
    }


    pub fn cursor_byte(&self) -> usize {
        self.buffer
            .char_indices()
            .nth(self.cursor)
            .map_or(self.buffer.len(), |(i, _)| i)
    }

    pub fn insert(&mut self, c: char) -> bool {
        if self.char_count() >= self.max_chars {
            return false;
        }
        let byte_idx = self.cursor_byte();
        self.buffer.insert(byte_idx, c);
        self.cursor += 1;
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let indices: Vec<_> = self.buffer.char_indices().collect();
        if let Some(&(byte_idx, _)) = indices.get(self.cursor - 1) {
            self.buffer.remove(byte_idx);
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    pub fn set_buffer(&mut self, text: String) {
        self.buffer = text;
        self.cursor = self.cursor.min(self.char_count());
    }
}
