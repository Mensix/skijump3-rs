use crate::store::{GameStateRef, ResourcesRef};
use crate::views::jump::competition::ui_state::{CompetitionUiState, ResultScreen};
use crate::views::jump::world_cup;
use engine::oxide::PaintCx;

pub(crate) enum CompetitionResultsRequest {
    Individual { ko_cursor_visible: bool },
}

pub(crate) fn render(
    cx: &mut PaintCx<'_>,
    resources: &ResourcesRef,
    state: &GameStateRef,
    ui_state: &CompetitionUiState,
    request: CompetitionResultsRequest,
) {
    match request {
        CompetitionResultsRequest::Individual { ko_cursor_visible } => {
            render_individual(cx, resources, state, ui_state, ko_cursor_visible)
        }
    }
}

fn render_individual(
    cx: &mut PaintCx<'_>,
    resources: &ResourcesRef,
    state: &GameStateRef,
    ui_state: &CompetitionUiState,
    ko_cursor_visible: bool,
) {
    state.borrow().active_competition.as_ref().map(|active| {
        let c = active.individual()?;
        match ui_state.current_screen() {
            ResultScreen::KoPairs(show_results) => world_cup::results::render_ko_pairs(
                cx,
                c,
                resources,
                show_results,
                ko_cursor_visible,
            ),
            ResultScreen::Stats => {
                world_cup::results::render_stats_page(cx, c, resources, ui_state.current_page())
            }
            ResultScreen::List => {
                let page_data = if ui_state.is_compact() {
                    world_cup::results::build_compact_results_page(c)
                } else {
                    world_cup::results::build_results_page(c, ui_state.current_page())
                };
                world_cup::results::render_results_page(cx, &page_data, resources);
                world_cup::results::render_header(cx, c, resources);
            }
        }
        Some(())
    });
}
