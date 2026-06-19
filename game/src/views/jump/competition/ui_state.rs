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
        Self::new_with_compact(false)
    }

    pub fn new_with_compact(compact: bool) -> Self {
        Self {
            render_mode: Cell::new(RenderMode::Jump),
            result_screen: Cell::new(ResultScreen::List),
            display_page: Cell::new(0),
            compact_list: Cell::new(compact),
            result_acknowledged: Cell::new(false),
            outcome_recorded: Cell::new(false),
            first_human_onbar: Cell::new(true),
            error_message: RefCell::new(String::new()),
        }
    }

    pub fn enter_error(&self, msg: String) {
        self.render_mode.set(RenderMode::Error);
        *self.error_message.borrow_mut() = msg;
    }

    #[must_use]
    pub fn error_message(&self) -> String {
        self.error_message.borrow().clone()
    }

    pub fn render_mode(&self) -> RenderMode {
        self.render_mode.get()
    }

    pub fn is_result_acknowledged(&self) -> bool {
        self.result_acknowledged.get()
    }

    pub fn enter_jump(&self) {
        self.render_mode.set(RenderMode::Jump);
        self.result_screen.set(ResultScreen::List);
        self.display_page.set(0);
    }

    pub fn enter_results(&self) {
        self.render_mode.set(RenderMode::Results);
        self.display_page.set(0);
    }

    pub fn enter_done(&self) {
        self.render_mode.set(RenderMode::Done);
    }

    pub fn acknowledge_outcome(&self) {
        self.result_acknowledged.set(true);
    }

    pub fn reset_acknowledged(&self) {
        self.result_acknowledged.set(false);
    }

    pub fn is_outcome_recorded(&self) -> bool {
        self.outcome_recorded.get()
    }

    pub fn reset_outcome_recorded(&self) {
        self.outcome_recorded.set(false);
    }

    pub fn mark_outcome_recorded(&self) {
        self.result_acknowledged.set(false);
        self.outcome_recorded.set(true);
        self.first_human_onbar.set(false);
    }

    pub fn is_first_human_onbar(&self) -> bool {
        self.first_human_onbar.get()
    }

    pub fn has_page(&self) -> bool {
        self.display_page.get() > 0
    }

    pub fn current_page(&self) -> usize {
        self.display_page.get()
    }

    pub fn next_page(&self, last_page: usize) -> bool {
        let page = self.display_page.get();
        if page + 1 < last_page {
            self.display_page.set(page + 1);
            true
        } else {
            false
        }
    }

    pub fn prev_page(&self) {
        let page = self.display_page.get();
        if page > 0 {
            self.display_page.set(page - 1);
        }
    }

    pub fn dismiss_results(&self) {
        self.display_page.set(0);
        self.result_screen.set(ResultScreen::List);
    }

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

    pub fn select_stats_screen(&self) {
        self.result_screen.set(ResultScreen::Stats);
        self.display_page.set(0);
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
