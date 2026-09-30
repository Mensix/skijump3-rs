use crate::gfx::theme::FONT_GRAY;
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;
use crate::ui::{Key, UiEvent};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageCursor(usize);

impl PageCursor {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn current(self) -> usize {
        self.0
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn apply(
        &mut self,
        event: PageEvent,
        total_pages: usize,
        dismissal: PageDismissal,
    ) -> bool {
        let total_pages = total_pages.max(1);
        self.0 = self.0.min(total_pages - 1);
        match event {
            PageEvent::First => self.reset(),
            PageEvent::Previous => self.0 = self.0.saturating_sub(1),
            PageEvent::Next if self.0 + 1 < total_pages => self.0 += 1,
            PageEvent::Next => return dismissal.past_end,
            PageEvent::Confirm => {
                return match dismissal.confirm {
                    ConfirmDismissal::Never => false,
                    ConfirmDismissal::Always => true,
                    ConfirmDismissal::LastPage => self.0 + 1 == total_pages,
                };
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageEvent {
    First,
    Previous,
    Next,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageEventMap {
    Individual,
    Koth,
    Team,
    Records,
}

pub fn page_event(event: UiEvent, map: PageEventMap) -> Option<PageEvent> {
    match (map, event) {
        (
            PageEventMap::Records
            | PageEventMap::Individual
            | PageEventMap::Koth
            | PageEventMap::Team,
            UiEvent::KeyDown(Key::Home),
        ) => Some(PageEvent::First),
        (PageEventMap::Koth, UiEvent::KeyDown(Key::Left | Key::Up | Key::PageUp))
        | (PageEventMap::Records, UiEvent::KeyDown(Key::Left | Key::PageUp))
        | (PageEventMap::Individual, UiEvent::KeyDown(Key::Left | Key::Up | Key::PageUp))
        | (PageEventMap::Team, UiEvent::KeyDown(Key::Left)) => Some(PageEvent::Previous),
        (
            PageEventMap::Koth,
            UiEvent::KeyDown(Key::Right | Key::Down | Key::PageDown) | UiEvent::Text(' '),
        )
        | (
            PageEventMap::Records,
            UiEvent::KeyDown(Key::Right | Key::PageDown | Key::Enter) | UiEvent::Text(' '),
        )
        | (
            PageEventMap::Individual,
            UiEvent::KeyDown(Key::Right | Key::Down | Key::PageDown) | UiEvent::Text(' '),
        )
        | (PageEventMap::Team, UiEvent::KeyDown(Key::Right) | UiEvent::Text(' ')) => {
            Some(PageEvent::Next)
        }
        (
            PageEventMap::Individual | PageEventMap::Koth | PageEventMap::Team,
            UiEvent::KeyDown(Key::Enter),
        ) => Some(PageEvent::Confirm),
        _ => None,
    }
}

pub fn is_quit_event(event: UiEvent) -> bool {
    match event {
        UiEvent::KeyDown(Key::F10) => true,
        UiEvent::TextWithModifiers('x' | 'X', modifiers) => modifiers.alt,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmDismissal {
    Never,
    Always,
    LastPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageDismissal {
    pub past_end: bool,
    pub confirm: ConfirmDismissal,
}

impl PageDismissal {
    pub const INDIVIDUAL: Self = Self {
        past_end: true,
        confirm: ConfirmDismissal::Always,
    };
    pub const KOTH: Self = Self {
        past_end: true,
        confirm: ConfirmDismissal::LastPage,
    };
    pub const TEAM: Self = Self {
        past_end: false,
        confirm: ConfirmDismissal::Always,
    };
    pub const RECORDS: Self = Self {
        past_end: true,
        confirm: ConfirmDismissal::Never,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageHintLayout {
    Top,
    Bottom,
}

pub fn render_page_hints(
    cx: &mut dyn UiCanvas,
    page: usize,
    total_pages: usize,
    lang: &LangBase,
    layout: PageHintLayout,
) {
    let last = page + 1 >= total_pages.max(1);
    let next = if last { lang.tr(248) } else { lang.tr(247) };
    match layout {
        PageHintLayout::Top => {
            if page > 0 {
                cx.right_text((319, 5), FONT_GRAY, &format!("(-{}", lang.tr(246)));
            }
            cx.right_text((319, 13), FONT_GRAY, &format!("{next}-)"));
        }
        PageHintLayout::Bottom => {
            if page > 0 {
                cx.text((3, 190), FONT_GRAY, &format!("<- {}", lang.tr(246)));
            }
            cx.right_text((319, 190), FONT_GRAY, &format!("{next} ->"));
        }
    }
}

pub fn differential_score(leader: f64, score: f64, rank: usize, enabled: bool) -> f64 {
    if enabled && rank > 1 {
        -(leader - score)
    } else {
        score
    }
}

pub fn cycle_index(current: usize, len: usize, dir: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i64;
    let cur_i = current as i64;
    let result = (cur_i + i64::from(dir)).rem_euclid(len_i);
    result as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Modifiers;

    #[test]
    fn cursor_preserves_dismissal_policies() {
        let mut cursor = PageCursor::default();
        assert!(!cursor.apply(PageEvent::Next, 2, PageDismissal::KOTH));
        assert_eq!(cursor.current(), 1);
        assert!(cursor.apply(PageEvent::Next, 2, PageDismissal::KOTH));

        cursor.reset();
        assert!(!cursor.apply(PageEvent::Confirm, 2, PageDismissal::KOTH));
        assert!(cursor.apply(PageEvent::Confirm, 2, PageDismissal::INDIVIDUAL));
        assert!(!cursor.apply(PageEvent::Next, 1, PageDismissal::TEAM));
    }

    #[test]
    fn event_maps_keep_screen_specific_keys() {
        assert_eq!(
            page_event(UiEvent::KeyDown(Key::Down), PageEventMap::Koth),
            Some(PageEvent::Next)
        );
        assert_eq!(
            page_event(UiEvent::KeyDown(Key::Down), PageEventMap::Individual),
            Some(PageEvent::Next)
        );
        assert_eq!(
            page_event(UiEvent::KeyDown(Key::Enter), PageEventMap::Records),
            Some(PageEvent::Next)
        );
    }

    #[test]
    fn team_and_koth_can_return_to_first_page() {
        for map in [PageEventMap::Team, PageEventMap::Koth] {
            assert_eq!(
                page_event(UiEvent::KeyDown(Key::Home), map),
                Some(PageEvent::First)
            );
        }
    }

    #[test]
    fn quit_shortcuts_require_their_explicit_modifier() {
        assert!(!is_quit_event(UiEvent::Text('x')));
        assert!(!is_quit_event(UiEvent::Text('X')));
        assert!(is_quit_event(UiEvent::TextWithModifiers(
            'x',
            Modifiers {
                ctrl: false,
                alt: true,
            },
        )));
        assert!(is_quit_event(UiEvent::KeyDown(Key::F10)));
    }

    #[test]
    fn cycle_forward_and_backward() {
        assert_eq!(cycle_index(4, 5, 1), 0);
        assert_eq!(cycle_index(0, 5, -1), 4);
        assert_eq!(cycle_index(0, 0, 1), 0);
        assert_eq!(cycle_index(0, 5, 7), 2);
    }

    #[test]
    fn differential_keeps_leader_and_replaces_following_scores_with_gap() {
        assert_eq!(differential_score(250.0, 250.0, 1, true), 250.0);
        assert_eq!(differential_score(250.0, 225.5, 2, true), -24.5);
        assert_eq!(differential_score(250.0, 225.5, 2, false), 225.5);
    }
}
