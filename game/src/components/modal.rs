use crate::gfx::theme::{BG_PURPLE, BG_RED, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::text::lang::LangBase;
use crate::ui::{Blinker, Key, UiCanvas, UiEvent};
use engine::color::Rgba;

pub const ALERT_RECT: (i32, i32, i32, i32) = (60, 80, 201, 51);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationChoice {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalResult {
    Dismissed,
    Confirmed(ConfirmationChoice),
}

#[derive(Debug)]
pub enum Modal {
    Alert {
        line1: String,
        line2: String,
    },
    ProfileMismatch {
        lines: [String; 4],
    },
    Confirm {
        line1: String,
        line2: String,
        blinker: Blinker,
    },
    Result {
        phase: u8,
        result: i32,
    },
}

impl Modal {
    pub fn alert(line1: impl Into<String>, line2: impl Into<String>) -> Self {
        Self::Alert {
            line1: line1.into(),
            line2: line2.into(),
        }
    }

    pub fn confirm(line1: impl Into<String>, line2: impl Into<String>) -> Self {
        Self::Confirm {
            line1: line1.into(),
            line2: line2.into(),
            blinker: Blinker::new(),
        }
    }

    pub fn result(phase: u8, result: i32) -> Self {
        Self::Result { phase, result }
    }

    pub fn hill_profile_mismatch(front_index: &str, exiting_cup: bool) -> Self {
        let third = if exiting_cup {
            "EXITING CUP."
        } else {
            "RUN THE HILL MAKER AGAIN."
        };
        Self::ProfileMismatch {
            lines: [
                format!("THE PROFILE OF THE HILL (FRONT{}.PCX)", front_index),
                "HAS BEEN CHANGED! NOT GOOD.".to_string(),
                third.to_string(),
                "PRESS A KEY...".to_string(),
            ],
        }
    }

    pub fn event(&self, event: UiEvent, lang: &LangBase) -> Option<ModalResult> {
        match (self, event) {
            (Self::Confirm { .. }, UiEvent::KeyDown(Key::Escape)) => Some(ModalResult::Dismissed),
            (Self::Confirm { .. }, UiEvent::Text(c)) => {
                confirmation_choice(c, lang).map(ModalResult::Confirmed)
            }
            (
                Self::Alert { .. } | Self::Result { .. } | Self::ProfileMismatch { .. },
                UiEvent::KeyDown(_),
            )
            | (
                Self::Alert { .. } | Self::Result { .. } | Self::ProfileMismatch { .. },
                UiEvent::Text(_),
            )
            | (
                Self::Alert { .. } | Self::Result { .. } | Self::ProfileMismatch { .. },
                UiEvent::TextWithModifiers(_, _),
            ) => Some(ModalResult::Dismissed),
            _ => None,
        }
    }

    pub fn paint(&self, cx: &mut dyn UiCanvas, lang: &LangBase) {
        match self {
            Self::Alert { line1, line2 } => {
                alert_box(cx, ALERT_RECT, BG_RED);
                cx.text((80, 90), FONT_GOLD, line1);
                cx.text((80, 105), FONT_GOLD, line2);
                cx.text((80, 120), FONT_GOLD, lang.tr(15));
            }
            Self::ProfileMismatch { lines } => {
                cx.fill((0, 0, 320, 200), BLACK);
                alert_box(cx, ALERT_RECT, BG_RED);
                cx.text((80, 85), FONT_GOLD, &lines[0]);
                cx.text((80, 95), FONT_GOLD, &lines[1]);
                cx.text((80, 105), FONT_GOLD, &lines[2]);
                cx.text((80, 120), FONT_GOLD, &lines[3]);
            }
            Self::Confirm {
                line1,
                line2,
                blinker,
            } => {
                alert_box(cx, ALERT_RECT, BG_RED);
                cx.text((80, 90), FONT_GOLD, line1);
                cx.text((80, 110), FONT_GOLD, line2);
                let choices = format!(" ({}/{})", lang.tr(6), lang.tr(7));
                let prompt_x = 80 + cx.string_width(line2) as i32;
                cx.text((prompt_x, 110), FONT_GRAY, &choices);
                let cursor_x = prompt_x + cx.string_width(&choices) as i32 + 8;
                cx.fill((cursor_x - 2, 108, 9, 11), BG_PURPLE);
                if blinker.visible(11, 10) {
                    cx.fill((cursor_x, 116, 5, 1), FONT_BODY);
                }
            }
            Self::Result { phase, result } => {
                alert_box(cx, (60, 80, 201, 41), BG_PURPLE);
                let index = 348 + (*phase as usize * 2) + usize::from(*result != 0);
                cx.text((75, 90), FONT_GOLD, lang.tr(index));
            }
        }
    }
}

pub fn confirmation_choice(c: char, lang: &LangBase) -> Option<ConfirmationChoice> {
    let matches = |translation: usize| {
        lang.tr(translation)
            .chars()
            .next()
            .is_some_and(|choice| c.eq_ignore_ascii_case(&choice))
    };
    if matches(6) {
        Some(ConfirmationChoice::Yes)
    } else if matches(7) {
        Some(ConfirmationChoice::No)
    } else {
        None
    }
}

pub fn alert_box(cx: &mut dyn UiCanvas, rect: (i32, i32, i32, i32), bg: Rgba) {
    cx.fill((rect.0 - 1, rect.1 - 1, rect.2 + 2, rect.3 + 2), BLACK);
    cx.pattern_fill(rect, bg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_accepts_only_localized_initials() {
        let mut strings = vec![String::new(); 8];
        strings[6] = "Tak".to_string();
        strings[7] = "Nie".to_string();
        let lang = LangBase::new(vec![strings], Vec::new(), 0);

        assert_eq!(
            confirmation_choice('t', &lang),
            Some(ConfirmationChoice::Yes)
        );
        assert_eq!(confirmation_choice('Y', &lang), None);
        assert_eq!(
            confirmation_choice('n', &lang),
            Some(ConfirmationChoice::No)
        );
        assert_eq!(confirmation_choice('x', &lang), None);
    }

    #[test]
    fn modal_confirm_handles_localized_text_and_escape() {
        let mut strings = vec![String::new(); 8];
        strings[6] = "Tak".to_string();
        strings[7] = "Nie".to_string();
        let lang = LangBase::new(vec![strings], Vec::new(), 0);
        let modal = Modal::confirm("Question", "Continue?");

        assert_eq!(
            modal.event(UiEvent::Text('t'), &lang),
            Some(ModalResult::Confirmed(ConfirmationChoice::Yes))
        );
        assert_eq!(
            modal.event(UiEvent::Text('n'), &lang),
            Some(ModalResult::Confirmed(ConfirmationChoice::No))
        );
        assert_eq!(
            modal.event(UiEvent::KeyDown(Key::Escape), &lang),
            Some(ModalResult::Dismissed)
        );
    }

    #[test]
    fn alert_dismisses_on_text_and_escape() {
        let lang = LangBase::new(vec![vec![String::new(); 8]], Vec::new(), 0);
        let modal = Modal::alert("Notice", "Done");

        assert_eq!(
            modal.event(UiEvent::Text('y'), &lang),
            Some(ModalResult::Dismissed)
        );
        assert_eq!(
            modal.event(UiEvent::KeyDown(Key::Escape), &lang),
            Some(ModalResult::Dismissed)
        );
    }

    #[test]
    fn alert_and_result_dismiss_on_any_key() {
        let lang = LangBase::new(vec![vec![String::new(); 356]], Vec::new(), 0);
        assert_eq!(
            Modal::alert("Notice", "Done").event(UiEvent::Text('x'), &lang),
            Some(ModalResult::Dismissed)
        );
        assert_eq!(
            Modal::result(2, 1).event(UiEvent::KeyDown(Key::Enter), &lang),
            Some(ModalResult::Dismissed)
        );
    }
}
