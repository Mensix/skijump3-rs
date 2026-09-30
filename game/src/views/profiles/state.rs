use crate::ui::{NumericSelector, TextInput};

pub(super) const REPLACE_MAX: usize = 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextField {
    Name,
    RealName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColorField {
    Suit,
    Ski,
}

#[derive(Debug)]
pub(super) enum Mode {
    List,
    Edit {
        profile: usize,
    },
    TextInput {
        profile: usize,
        field: TextField,
        input: TextInput,
    },
    ColorSelect {
        profile: usize,
        field: ColorField,
        selector: NumericSelector,
        color_x: i32,
        color_y: i32,
        color_max: usize,
        color_suit: bool,
    },
    ReplaceSelect {
        profile: usize,
        selector: NumericSelector,
    },
}

pub(super) enum Pending {
    EditEnter(usize, usize),
    TextCommit(usize, TextField, String),
    TextCancel(usize, TextField),
    ColorCommit(usize, ColorField),
    ColorCancel(usize, ColorField),
    ReplaceCommit(usize),
    ReplaceCancel(usize),
}
