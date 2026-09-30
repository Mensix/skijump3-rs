use crate::competition::types::CompetitionPhase;
use crate::components::page_nav::{PageCursor, PageDismissal, PageEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Jump,
    Results,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultScreen {
    List,
    KoPairs(bool),
    Stats,
}

#[derive(Debug, Clone)]
pub struct CompetitionUiState {
    render_mode: RenderMode,
    result_screen: ResultScreen,
    page: PageCursor,
    compact_list: bool,
    result_acknowledged: bool,
    outcome_recorded: bool,
    first_human_onbar: bool,
    cup_exit_requested: bool,
}

impl CompetitionUiState {
    pub fn new() -> Self {
        Self::new_with_compact(false)
    }

    pub fn new_with_compact(compact: bool) -> Self {
        Self {
            render_mode: RenderMode::Jump,
            result_screen: ResultScreen::List,
            page: PageCursor::default(),
            compact_list: compact,
            result_acknowledged: false,
            outcome_recorded: false,
            first_human_onbar: true,
            cup_exit_requested: false,
        }
    }

    pub fn request_cup_exit(&mut self) {
        self.cup_exit_requested = true;
    }

    pub fn take_cup_exit_request(&mut self) -> bool {
        std::mem::replace(&mut self.cup_exit_requested, false)
    }

    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    pub fn is_result_acknowledged(&self) -> bool {
        self.result_acknowledged
    }

    pub fn enter_jump(&mut self) {
        self.render_mode = RenderMode::Jump;
        self.result_screen = ResultScreen::List;
        self.page.reset();
    }

    pub fn enter_results(&mut self) {
        self.render_mode = RenderMode::Results;
        self.page.reset();
    }

    pub fn enter_done(&mut self) {
        self.render_mode = RenderMode::Done;
    }

    pub fn acknowledge_outcome(&mut self) {
        self.result_acknowledged = true;
    }

    pub fn reset_acknowledged(&mut self) {
        self.result_acknowledged = false;
    }

    pub fn is_outcome_recorded(&self) -> bool {
        self.outcome_recorded
    }

    pub fn reset_outcome_recorded(&mut self) {
        self.outcome_recorded = false;
    }

    pub fn mark_outcome_recorded(&mut self) {
        self.result_acknowledged = false;
        self.outcome_recorded = true;
        self.first_human_onbar = false;
    }

    pub fn is_first_human_onbar(&self) -> bool {
        self.first_human_onbar
    }

    pub fn has_page(&self) -> bool {
        self.page.current() > 0
    }

    pub fn current_page(&self) -> usize {
        self.page.current()
    }

    pub fn handle_page_event(
        &mut self,
        event: PageEvent,
        total_pages: usize,
        dismissal: PageDismissal,
    ) -> bool {
        self.page.apply(event, total_pages, dismissal)
    }

    pub fn dismiss_results(&mut self) {
        self.page.reset();
        self.result_screen = ResultScreen::List;
    }

    pub fn is_compact(&self) -> bool {
        self.compact_list
    }

    pub fn toggle_compact(&mut self) {
        self.compact_list = !self.compact_list;
        self.result_screen = ResultScreen::List;
        self.page.reset();
    }

    pub fn toggle_stats(&mut self) {
        self.result_screen = match self.result_screen {
            ResultScreen::Stats => ResultScreen::List,
            _ => ResultScreen::Stats,
        };
        self.page.reset();
    }

    pub fn toggle_ko_pairs(&mut self, show_results: bool) {
        self.result_screen = match self.result_screen {
            ResultScreen::KoPairs(_) => ResultScreen::List,
            _ => ResultScreen::KoPairs(show_results),
        };
    }

    pub fn select_default_screen(&mut self, is_four_hills: bool, phase: CompetitionPhase) {
        if is_four_hills {
            self.result_screen = if phase == CompetitionPhase::QualificationResults {
                ResultScreen::KoPairs(false)
            } else if phase == CompetitionPhase::Round1Results {
                ResultScreen::KoPairs(true)
            } else {
                ResultScreen::List
            };
        } else {
            self.result_screen = ResultScreen::List;
        }
    }

    pub fn select_stats_screen(&mut self) {
        self.result_screen = ResultScreen::Stats;
        self.page.reset();
    }

    pub fn current_screen(&self) -> ResultScreen {
        self.result_screen
    }
}

impl Default for CompetitionUiState {
    fn default() -> Self {
        Self::new()
    }
}
