use std::cell::UnsafeCell;

pub struct State<T> {
    value: UnsafeCell<T>,
    dirty: UnsafeCell<bool>,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        Self { value: UnsafeCell::new(value), dirty: UnsafeCell::new(true) }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    pub fn set(&self, value: T) {
        unsafe { *self.value.get() = value; }
        unsafe { *self.dirty.get() = true; }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { *self.dirty.get() }
    }

    pub fn mark_clean(&self) {
        unsafe { *self.dirty.get() = false; }
    }
}

unsafe impl<T> Send for State<T> {}
unsafe impl<T> Sync for State<T> {}