use std::cell::Cell;

#[derive(Debug)]
pub struct Blinker {
    counter: Cell<u32>,
}

impl Blinker {
    pub fn new() -> Self {
        Self {
            counter: Cell::new(0),
        }
    }

    pub fn reset(&self) {
        self.counter.set(0);
    }

    pub fn visible(&self, on: u32, off: u32) -> bool {
        let c = self.counter.get();
        self.counter.set(c + 1);
        c % (on + off) < on
    }
}

impl Default for Blinker {
    fn default() -> Self {
        Self::new()
    }
}
