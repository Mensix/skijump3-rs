pub mod element;
pub mod paint;
mod router;
mod text;

pub use element::{Element, ImageRegion};
pub use paint::PaintCtx;
pub use router::{Event, Key, Router, View};
pub use text::Font;

pub trait Component {
    type Action;

    fn elements(&self) -> Vec<Element>;
    fn handle_event(&mut self, event: &Event) -> Option<Self::Action>;
}
