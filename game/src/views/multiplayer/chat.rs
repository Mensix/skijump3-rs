use engine::oxide::input::{Key, UiEvent};
use engine::oxide::paint::PaintCx;
use engine::oxide::widget::EventCx;
use engine::oxide::{Blinker, TextEditState};

use crate::gfx::theme::{BLACK, FILL_DARK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};

use super::state::ChatBuffer;

const CHAT_X: i32 = 4;
const CHAT_Y: i32 = 156;
const CHAT_W: i32 = 312;
const CHAT_H: i32 = 40;

pub fn paint_chat(cx: &mut PaintCx<'_>, chat: &ChatBuffer, input: &str, blinker: &Blinker) {
    cx.fill((CHAT_X, CHAT_Y, CHAT_W, CHAT_H), FILL_DARK);
    cx.stroke((CHAT_X, CHAT_Y, CHAT_W, CHAT_H), FILL_GRAY);
    cx.fill((CHAT_X + 1, CHAT_Y + 1, CHAT_W - 2, 9), BLACK);
    cx.text((CHAT_X + 6, CHAT_Y + 2), FONT_GOLD, "CHAT");
    cx.right_text((CHAT_X + CHAT_W - 6, CHAT_Y + 2), FONT_GRAY, "Enter sends");

    let prompt = "> ";
    let pw = cx.string_width(prompt) as i32;
    cx.text((CHAT_X + 6, CHAT_Y + 12), FONT_TEAL, prompt);
    cx.text((CHAT_X + 6 + pw, CHAT_Y + 12), FONT_BODY, input);

    if blinker.visible(11, 10) {
        let cursor_x = CHAT_X + 6 + pw + cx.string_width(input) as i32;
        cx.fill((cursor_x, CHAT_Y + 18, 5, 1), FONT_GOLD);
    }

    let msg_start = CHAT_Y + 23;
    let n = chat.messages.len();
    let visible = 2;
    if n > 0 {
        let start = n.saturating_sub(visible);
        for i in 0..visible.min(n) {
            let idx = start + i;
            let (sender, text) = &chat.messages[idx];
            let y = msg_start + i as i32 * 8;
            cx.text((CHAT_X + 6, y), FONT_GOLD, sender);
            cx.text((CHAT_X + 42, y), FONT_BODY, text);
        }
    } else {
        cx.text(
            (CHAT_X + 6, msg_start),
            FONT_GRAY,
            "Type immediately. No focus switching.",
        );
        cx.text(
            (CHAT_X + 6, msg_start + 8),
            FONT_GRAY,
            "F1 Ready  F2 Start(host)  Esc Leave",
        );
    }
}

pub fn handle_chat_event(
    chat: &mut ChatBuffer,
    input: &mut TextEditState,
    event: UiEvent,
    ecx: &mut EventCx,
    on_send: &mut impl FnMut(String),
) {
    match event {
        UiEvent::KeyDown(Key::Backspace) => {
            input.backspace();
            ecx.consume();
        }
        UiEvent::KeyDown(Key::Enter) => {
            let text = input.buffer().to_string();
            if !text.is_empty() {
                chat.send(text.clone());
                on_send(text);
                input.set_buffer(String::new());
            }
            ecx.consume();
        }
        UiEvent::Text(c) if c >= ' ' => {
            input.insert(c);
            ecx.consume();
        }
        _ => {}
    }
}
