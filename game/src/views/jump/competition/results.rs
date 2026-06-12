use crate::competition::team_cup::types::TeamCupResultsKind;
use crate::components::screen;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::ui_state::{CompetitionUiState, ResultScreen};
use crate::views::jump::team_cup;
use crate::views::jump::world_cup;
use engine::ui::Element;

pub(crate) enum CompetitionResultsRequest {
    Individual { ko_cursor_visible: bool },
    TeamCup { kind: TeamCupResultsKind },
    Koth,
}

pub(crate) fn render(
    resources: &ResourcesRef,
    store: &StoreRef,
    ui_state: &CompetitionUiState,
    request: CompetitionResultsRequest,
) -> Vec<Element> {
    match request {
        CompetitionResultsRequest::Individual { ko_cursor_visible } => {
            render_individual(resources, store, ui_state, ko_cursor_visible)
        }
        CompetitionResultsRequest::TeamCup { kind } => {
            team_cup::results::render(resources, store, kind)
        }
        CompetitionResultsRequest::Koth => {
            crate::views::jump::koth::results::render(resources, store)
        }
    }
}

fn render_individual(
    resources: &ResourcesRef,
    store: &StoreRef,
    ui_state: &CompetitionUiState,
    ko_cursor_visible: bool,
) -> Vec<Element> {
    store
        .with_active(|active| {
            let c = active.individual()?;
            Some(match ui_state.current_screen() {
                ResultScreen::KoPairs(show_results) => world_cup::results::render_ko_pairs(
                    c,
                    resources,
                    show_results,
                    ko_cursor_visible,
                ),
                ResultScreen::Stats => {
                    world_cup::results::render_stats_page(c, resources, ui_state.current_page())
                }
                ResultScreen::List => {
                    let page_data = if ui_state.is_compact() {
                        world_cup::results::build_compact_results_page(c)
                    } else {
                        world_cup::results::build_results_page(c, ui_state.current_page())
                    };
                    let mut els = world_cup::results::render_results_page(&page_data, resources);
                    els.extend(world_cup::results::render_header(c, resources));
                    els
                }
            })
        })
        .flatten()
        .unwrap_or_else(screen::black_screen)
}
