use crate::ui::{Component, Element, Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    text: String,
    x: i32,
    y: i32,
    color: u8,
    align: Align,
}

impl Cell {
    pub fn left(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            color,
            align: Align::Left,
        }
    }

    pub fn right(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            color,
            align: Align::Right,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    cells: Vec<Cell>,
}

impl Table {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn push(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn into_elements(self) -> Vec<Element> {
        self.cells
            .into_iter()
            .map(|cell| match cell.align {
                Align::Left => Element::text_color(cell.text, cell.x, cell.y, cell.color),
                Align::Right => Element::text_color_right(cell.text, cell.x, cell.y, cell.color),
            })
            .collect()
    }
}

impl Component for Table {
    type Action = ();

    fn elements(&self) -> Vec<Element> {
        self.cells
            .iter()
            .map(|cell| match cell.align {
                Align::Left => Element::text_color(cell.text.clone(), cell.x, cell.y, cell.color),
                Align::Right => {
                    Element::text_color_right(cell.text.clone(), cell.x, cell.y, cell.color)
                }
            })
            .collect()
    }

    fn handle_event(&mut self, _event: &Event) -> Option<Self::Action> {
        None
    }
}
