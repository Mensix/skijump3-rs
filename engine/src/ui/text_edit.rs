/// Pure-data text-editing model with char-index cursor.
///
/// Tracks cursor as a character offset (not byte offset) for Pascal‑compatible
/// semantics where one char == one visual position.
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

    /// Byte index of the cursor position, for slicing the buffer.
    pub fn cursor_byte(&self) -> usize {
        self.buffer
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.buffer.len())
    }

    /// Insert `c` at the cursor position.
    /// Returns `false` if the char-count limit would be exceeded.
    pub fn insert(&mut self, c: char) -> bool {
        if self.char_count() >= self.max_chars {
            return false;
        }
        let byte_idx = self.cursor_byte();
        self.buffer.insert(byte_idx, c);
        self.cursor += 1;
        true
    }

    /// Delete the character immediately before the cursor.
    /// Returns `false` if the cursor is already at position 0.
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
