pub mod blinker;
pub mod element;
mod router;
pub mod selection;
pub mod table;
pub mod text;
pub mod text_edit;

pub use crate::bitmap::IndexedBitmap;
pub use blinker::Blinker;
pub use element::{Element, ImageRegion};
pub use router::{BackgroundMode, Event, Key, Router, View};
pub use selection::SelectionState;
pub use table::{Align, Cell, Table};
pub use text::Font;
pub use text_edit::TextEditState;

pub trait Component {
    type Action;

    fn elements(&self) -> Vec<Element>;
    fn handle_event(&mut self, event: &Event) -> Option<Self::Action>;
}
