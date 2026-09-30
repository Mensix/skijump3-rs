#[derive(Debug, Default)]
pub(crate) struct CompletionSaveGuard {
    saved: bool,
}

impl CompletionSaveGuard {
    pub(crate) fn claim(&mut self) -> bool {
        if self.saved {
            return false;
        }
        self.saved = true;
        true
    }
}
