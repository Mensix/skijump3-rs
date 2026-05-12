mod text;
mod router;
pub mod element;
pub mod paint;

pub use paint::PaintCtx;
pub use text::Font;
pub use router::{View, Event, Key, Router};
pub use element::Element;

pub trait Component {
    fn elements(&self) -> Vec<Element>;
    fn handle_event(&mut self, event: &Event) -> Option<usize>;
}
