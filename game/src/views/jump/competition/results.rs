use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::views::jump::competition::ui_state::{CompetitionUiState, ResultScreen};
use crate::views::jump::world_cup;

pub(crate) fn render(
    cx: &mut dyn UiCanvas,
    resources: &ResourcesRef,
    state: &GameState,
    ui_state: &CompetitionUiState,
    ko_cursor_visible: bool,
    hill_background: bool,
) {
    render_individual(
        cx,
        resources,
        state,
        ui_state,
        ko_cursor_visible,
        hill_background,
    )
}

fn render_individual(
    cx: &mut dyn UiCanvas,
    resources: &ResourcesRef,
    state: &GameState,
    ui_state: &CompetitionUiState,
    ko_cursor_visible: bool,
    hill_background: bool,
) {
    state.active_competition.as_ref().map(|active| {
        let c = active.individual()?;
        match ui_state.current_screen() {
            ResultScreen::KoPairs(show_results) => world_cup::results::render_ko_pairs(
                cx,
                c,
                resources,
                show_results,
                ko_cursor_visible,
                hill_background,
            ),
            ResultScreen::Stats => world_cup::results::render_stats_page(
                cx,
                c,
                resources,
                ui_state.current_page(),
                hill_background,
            ),
            ResultScreen::List => {
                let page_data = if ui_state.is_compact() {
                    world_cup::results::build_compact_results_page(c)
                } else {
                    world_cup::results::build_results_page(c, ui_state.current_page())
                };
                world_cup::results::render_results_page(
                    cx,
                    &page_data,
                    resources,
                    state.config.event_gap != 0,
                    state.config.wc_gap != 0,
                    hill_background,
                );
                world_cup::results::render_header(cx, c, resources);
            }
        }
        Some(())
    });
}
