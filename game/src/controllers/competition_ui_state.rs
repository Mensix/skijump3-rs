use crate::competition::types::CompetitionPhase;
use std::cell::{Cell, RefCell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Jump,
    Results,
    Done,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultScreen {
    List,
    KoPairs(bool),
    Stats,
}

/// Consolidated UI state for a competition view.
///
/// Replaces the 7 scattered `Cell` fields in `WorldCupJumpView`.
/// All mutations go through named methods so the lifecycle is
/// auditable in one place.
#[derive(Debug, Clone)]
pub struct CompetitionUiState {
    render_mode: Cell<RenderMode>,
    result_screen: Cell<ResultScreen>,
    display_page: Cell<usize>,
    compact_list: Cell<bool>,
    result_acknowledged: Cell<bool>,
    outcome_recorded: Cell<bool>,
    first_human_onbar: Cell<bool>,
    error_message: RefCell<String>,
}

impl CompetitionUiState {
    pub fn new() -> Self {
        Self {
            render_mode: Cell::new(RenderMode::Jump),
            result_screen: Cell::new(ResultScreen::List),
            display_page: Cell::new(0),
            compact_list: Cell::new(false),
            result_acknowledged: Cell::new(false),
            outcome_recorded: Cell::new(false),
            first_human_onbar: Cell::new(true),
            error_message: RefCell::new(String::new()),
        }
    }

    /// Show a full-screen error message. The user can dismiss it
    /// with any key (routing back to main menu).
    pub fn enter_error(&self, msg: String) {
        self.render_mode.set(RenderMode::Error);
        *self.error_message.borrow_mut() = msg;
    }

    #[must_use]
    pub fn error_message(&self) -> String {
        self.error_message.borrow().clone()
    }

    // ── Getters ─────────────────────────────────────────────────

    pub fn render_mode(&self) -> RenderMode {
        self.render_mode.get()
    }

    pub fn is_result_acknowledged(&self) -> bool {
        self.result_acknowledged.get()
    }

    // ── Mode transitions ────────────────────────────────────────

    /// Show human jump scene.
    pub fn enter_jump(&self) {
        self.render_mode.set(RenderMode::Jump);
        self.result_screen.set(ResultScreen::List);
        self.display_page.set(0);
    }

    /// Show results/standings list.
    pub fn enter_results(&self) {
        self.render_mode.set(RenderMode::Results);
        self.display_page.set(0);
    }

    /// Season/batch is done.
    pub fn enter_done(&self) {
        self.render_mode.set(RenderMode::Done);
    }

    // ── Outcome acknowledgment ──────────────────────────────────

    /// Record that the user has seen the outcome (pressed Enter/Escape/DQ key).
    pub fn acknowledge_outcome(&self) {
        self.result_acknowledged.set(true);
    }

    /// Reset the acknowledged flag (for the next jumper).
    pub fn reset_acknowledged(&self) {
        self.result_acknowledged.set(false);
    }

    /// Whether the outcome has already been recorded into the competition store.
    pub fn is_outcome_recorded(&self) -> bool {
        self.outcome_recorded.get()
    }

    pub fn reset_outcome_recorded(&self) {
        self.outcome_recorded.set(false);
    }

    /// Mark the outcome as recorded (prevents double-recording on scene rebuild).
    pub fn mark_outcome_recorded(&self) {
        self.result_acknowledged.set(false);
        self.outcome_recorded.set(true);
        self.first_human_onbar.set(false);
    }

    // ── Overlay helpers ─────────────────────────────────────────

    pub fn is_first_human_onbar(&self) -> bool {
        self.first_human_onbar.get()
    }

    // ── Results paging ──────────────────────────────────────────

    pub fn has_page(&self) -> bool {
        self.display_page.get() > 0
    }

    pub fn current_page(&self) -> usize {
        self.display_page.get()
    }

    /// Advance to next page. Returns true if there was a next page.
    pub fn next_page(&self, last_page: usize) -> bool {
        let page = self.display_page.get();
        if page + 1 < last_page {
            self.display_page.set(page + 1);
            true
        } else {
            false
        }
    }

    /// Go to previous page.
    pub fn prev_page(&self) {
        let page = self.display_page.get();
        if page > 0 {
            self.display_page.set(page - 1);
        }
    }

    /// Reset display state after a results screen is dismissed.
    pub fn dismiss_results(&self) {
        self.display_page.set(0);
        self.result_screen.set(ResultScreen::List);
    }

    // ── Result screen toggles ───────────────────────────────────

    pub fn is_compact(&self) -> bool {
        self.compact_list.get()
    }

    pub fn toggle_compact(&self) {
        self.compact_list.set(!self.compact_list.get());
        self.result_screen.set(ResultScreen::List);
        self.display_page.set(0);
    }

    pub fn toggle_stats(&self) {
        self.result_screen.set(match self.result_screen.get() {
            ResultScreen::Stats => ResultScreen::List,
            _ => ResultScreen::Stats,
        });
        self.display_page.set(0);
    }

    pub fn toggle_ko_pairs(&self, show_results: bool) {
        self.result_screen.set(match self.result_screen.get() {
            ResultScreen::KoPairs(_) => ResultScreen::List,
            _ => ResultScreen::KoPairs(show_results),
        });
    }

    /// Select default screen based on competition style.
    pub fn select_default_screen(&self, is_four_hills: bool, phase: CompetitionPhase) {
        if is_four_hills {
            self.result_screen
                .set(if phase == CompetitionPhase::QualificationResults {
                    ResultScreen::KoPairs(false)
                } else if phase == CompetitionPhase::Round1Results {
                    ResultScreen::KoPairs(true)
                } else {
                    ResultScreen::List
                });
        } else {
            self.result_screen.set(ResultScreen::List);
        }
    }

    pub fn current_screen(&self) -> ResultScreen {
        self.result_screen.get()
    }
}

impl Default for CompetitionUiState {
    fn default() -> Self {
        Self::new()
    }
}
