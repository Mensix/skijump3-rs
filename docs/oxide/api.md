# Oxide API Proposal

This document sketches the intended Rust API. Names are provisional, but the architecture should stay close to this shape.

## Module Layout

```text
game/src/oxide/
  mod.rs
  adapter.rs
  ui.rs
  handler.rs
  screen.rs
  text.rs
  keymap.rs
  list.rs
  menu.rs
  form.rs
  modal.rs
  component.rs
```

## Public Prelude

`game/src/oxide/mod.rs` should re-export common items:

```rust
pub use adapter::OxideAdapter;
pub use component::{OxideComponent, ComponentMount};
pub use form::{Form, FormRow};
pub use keymap::KeyMap;
pub use list::SelectableList;
pub use menu::OxideMenu;
pub use modal::ConfirmModal;
pub use screen::ScreenStyle;
pub use ui::{OxideView, Ui, UiCtx};
```

## OxideView

```rust
pub trait OxideView {
    type Msg;
    type Route: Clone + PartialEq + 'static;

    fn build(&self, ui: &mut Ui<Self::Msg>);
    fn update(&mut self, msg: Self::Msg) -> Option<Self::Route>;

    fn update_frame(&mut self) {}
    fn apply_palette(&self, _: &mut Palette) {}
}
```

## Ui

Minimum viable API:

```rust
pub struct Ui<Msg> {
    elements: Vec<Element>,
    handlers: Vec<KeyHandler<Msg>>,
    modal_depth: usize,
}

impl<Msg> Ui<Msg> {
    pub fn new() -> Self;
    pub fn into_elements(self) -> Vec<Element>;
    pub fn dispatch(self, event: &Event) -> Option<Msg>;

    pub fn push(&mut self, element: Element);
    pub fn extend(&mut self, elements: impl IntoIterator<Item = Element>);

    pub fn text(&mut self, text: impl Into<String>, x: i32, y: i32, color: u8);
    pub fn right_text(&mut self, text: impl Into<String>, x: i32, y: i32, color: u8);
    pub fn center_text(&mut self, text: impl Into<String>, x: i32, y: i32, color: u8);
    pub fn fillbox(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8);
    pub fn box_(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8);
    pub fn fill_area(&mut self, thing: u8);
    pub fn sprite(&mut self, idx: u16, x: i32, y: i32);

    pub fn on_key(&mut self, key: Key, msg: Msg);
    pub fn on_keys(&mut self, keys: impl IntoIterator<Item = Key>, msg: Msg)
    where
        Msg: Clone;

    pub fn screen(&mut self, style: ScreenStyle);
    pub fn modal(&mut self, build: impl FnOnce(&mut Ui<Msg>));
}
```

## KeyHandler

```rust
pub enum HandlerScope {
    Main,
    Editor,
    Modal,
}

pub struct KeyHandler<Msg> {
    pub scope: HandlerScope,
    pub key: KeyMatch,
    pub msg: Msg,
}

pub enum KeyMatch {
    Exact(Key),
    Char(char),
    AnyChar,
}
```

Dispatch rule: modal handlers win over editor handlers, editor handlers win over main handlers, newer handlers win over older handlers within the same scope.

## KeyMap Builder

```rust
impl<Msg> Ui<Msg> {
    pub fn keys(&mut self) -> KeyMap<'_, Msg>;
}

impl<'a, Msg> KeyMap<'a, Msg> {
    pub fn up(self, msg: Msg) -> Self;
    pub fn down(self, msg: Msg) -> Self;
    pub fn left(self, msg: Msg) -> Self;
    pub fn right(self, msg: Msg) -> Self;
    pub fn enter(self, msg: Msg) -> Self;
    pub fn space(self, msg: Msg) -> Self;
    pub fn escape(self, msg: Msg) -> Self;
    pub fn delete(self, msg: Msg) -> Self;
}
```

`Msg: Clone` bounds can be on methods that register multiple keys.

## ScreenStyle

Typed replacement for numeric `new_screen(style)`:

```rust
pub enum ScreenStyle {
    Black,
    MenuTop,      // Pascal NewScreen(1,...)
    MenuSide,     // Pascal NewScreen(2,0)
    SplitRecords, // Pascal NewScreen(4,...)
    FullPanel,    // Pascal NewScreen(5,...)
    Profiles,
}
```

`Ui::screen(style)` appends the exact existing `Element`s. Keep comments with Pascal references.

## OxideComponent

Use components for reusable pieces. Components should emit messages but should not mutate parent state during build.

```rust
pub trait OxideComponent<Msg> {
    fn build(&self, ui: &mut Ui<Msg>);
}
```

Stateful editors can either:

- remain stored in the view state and register handlers through `build`, or
- implement a second trait for local dispatch.

Recommended initial approach: adapt current `engine::ui::Component` components.

## Legacy Component Adapter

Existing components (`Menu`, `TextInput`, `ValueSelector`, `ConfirmDialog`) can be bridged.

```rust
pub trait ComponentMount<C> {
    fn mount<Msg>(&self, ui: &mut Ui<Msg>, map: impl Fn(C::Action) -> Msg)
    where
        C: Component;
}
```

However, because `Component::handle_event` requires `&mut self`, a complete bridge needs a view-owned component plus manual `update(msg)` handling. Do not overbuild this first. Prefer rewriting small components into Oxide style as screens are ported.

## Menu API

```rust
ui.menu()
    .at(11, 97)
    .item(1, lang.lstr(20), Msg::Menu(1))
    .item(2, lang.lstr(21), Msg::Menu(2))
    .exit(lang.lstr(26), Msg::Back)
    .selected(self.selected)
    .box_color(FONT_DEFAULT)
    .text_color(FONT_DEFAULT)
    .build();
```

The menu builder should render labels, selection box, numeric hotkeys, Up/Down, Enter/Space, Escape.

## List API

```rust
ui.selectable_list("profiles")
    .selected(self.selected)
    .wrap(true)
    .row_height(8)
    .on_move(Msg::MoveList)
    .on_submit(Msg::OpenSelected)
    .on_delete(Msg::DeleteSelected)
    .rows(|rows| {
        rows.item("JUMPER 1", 40, 12, FONT_NAME);
        rows.item("*Create New Jumper*", 40, 148, FONT_NEW);
        rows.exit("Back to Main Menu", 40, 164, FONT_BACK);
    });
```

Do not force automatic layout. List can provide helpers for common row calculations but must allow exact x/y.

## Form API

```rust
ui.form("profile")
    .selected(edit_selected)
    .box_(162, 10, 155, 9, FONT_DEFAULT)
    .row("Name:", profile.name.clone(), 166, y, Msg::EditName)
    .row("Real name:", profile.real_name.clone(), 166, y, Msg::EditRealName)
    .readonly("Total Jumps:", profile.total_jumps.to_string(), 166, y)
    .build();
```

Form rows must support:

- label text
- value text
- dynamic value x based on label width
- selectable rows
- read-only/stat rows
- custom renderer for suit/ski swatches

## Modal API

```rust
ui.modal(|ui| {
    ui.modal_box(59, 79, 203, 53);
    ui.text(message, 70, 90, FONT_GOLD);
    ui.text("Are you sure?", 70, 110, FONT_GOLD);
    ui.text("(Y/N)", hint_x, 110, FONT_HELP);
    ui.keys()
        .char('y', Msg::Confirm)
        .char('Y', Msg::Confirm)
        .char('n', Msg::Cancel)
        .char('N', Msg::Cancel)
        .escape(Msg::Cancel);
});
```

Modal must block main handlers.
