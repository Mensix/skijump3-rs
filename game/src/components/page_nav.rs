use crate::route::RouteTarget;
use engine::ui::{Event, Key};

/// Cycle `current` by `dir` (±1 or any integer) wrapping around `[0, len)`.
/// Returns `0` when `len == 0`.
#[must_use]
pub fn cycle_index(current: usize, len: usize, dir: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i64;
    let cur_i = current as i64;
    let result = (cur_i + i64::from(dir)).rem_euclid(len_i);
    result as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageAction {
    Next,
    Prev,
    First,
    Back,
}

pub(crate) fn page_action(event: Event, page: usize, pages: usize) -> Option<PageAction> {
    match event {
        Event::Keyboard(Key::Escape) => Some(PageAction::Back),
        Event::Keyboard(Key::Home) => Some(PageAction::First),
        Event::Keyboard(Key::Left | Key::PageUp) if page > 0 => Some(PageAction::Prev),
        Event::Keyboard(Key::Right | Key::PageDown | Key::Enter | Key::Char(' ')) => {
            Some(PageAction::Next)
        }
        Event::Keyboard(_) => {
            let _ = pages;
            None
        }
    }
}

pub(crate) fn apply_page_action(
    action: PageAction,
    page: &mut usize,
    pages: usize,
) -> Option<RouteTarget> {
    match action {
        PageAction::Back => Some(RouteTarget::Back),
        PageAction::First => {
            *page = 0;
            None
        }
        PageAction::Prev => {
            *page = page.saturating_sub(1);
            None
        }
        PageAction::Next => {
            *page += 1;
            if *page >= pages {
                Some(RouteTarget::MainMenu)
            } else {
                None
            }
        }
    }
}

pub(crate) fn handle_paged_event(
    event: Event,
    page: &mut usize,
    pages: usize,
) -> Option<RouteTarget> {
    if *page >= pages {
        *page = pages.saturating_sub(1);
    }
    page_action(event, *page, pages).and_then(|action| apply_page_action(action, page, pages))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_forward() {
        assert_eq!(cycle_index(0, 5, 1), 1);
        assert_eq!(cycle_index(4, 5, 1), 0);
    }

    #[test]
    fn cycle_backward() {
        assert_eq!(cycle_index(0, 5, -1), 4);
        assert_eq!(cycle_index(4, 5, -1), 3);
    }

    #[test]
    fn empty_list() {
        assert_eq!(cycle_index(0, 0, 1), 0);
    }

    #[test]
    fn large_delta() {
        assert_eq!(cycle_index(0, 5, 7), 2);
        assert_eq!(cycle_index(0, 5, -7), 3);
    }
}
