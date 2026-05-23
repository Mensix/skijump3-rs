# Oxide Architecture

## Problem Statement

Current views are built directly from `Vec<Element>` plus ad hoc event matching. This is correct but too low-level for scalable UI authoring. Screens like Profiles are conceptually simple but spread across multiple files because the codebase lacks reusable concepts for focus, rows, forms, lists, modal overlays, and typed UI messages.

The problem is not that there are multiple files. The problem is that each view reimplements a mini UI framework.

## Design Goals

- Preserve exact Pascal-visible behavior.
- Keep `engine` generic and small.
- Make UI construction declarative and local.
- Make keyboard dispatch reusable and testable.
- Use typed messages instead of scattered side effects.
- Support modal stacking and focused editors.
- Reduce repeated `els.push(...)` boilerplate.
- Make new screens predictable to implement.

## Non-Goals

- No React clone.
- No virtual DOM.
- No retained tree diffing.
- No CSS/flexbox/layout engine.
- No procedural macros initially.
- No async UI lifecycle.
- No global reactive state.
- No immediate rewrite of all views.

## Layering

```text
engine::ui
  Element
  Event / Key
  View / Router
  Font
  TextEditState
  SelectionState

game::oxide
  Ui builder
  OxideView trait
  typed message dispatch
  focus and modal handling
  Pascal screen templates
  lists / menus / forms / modals

game::views
  screen state
  build(ui)
  update(msg)
```

`engine` remains the low-level renderer/runtime. Oxide is the game UI authoring layer.

## Runtime Model

Oxide is immediate-mode and message-based:

```text
1. View state exists in the screen struct.
2. build(&self, ui) emits Elements and registers event handlers.
3. elements() returns ui.into_elements().
4. handle_event(event) rebuilds the UI handler table for current state.
5. Ui dispatches the event to the active modal/focus scope.
6. A typed Msg is produced.
7. update(msg) mutates state and optionally returns a route.
```

Rebuilding for event dispatch is acceptable. The game runs at 320x200 with small UI trees. Correctness and simplicity are more valuable than retaining a complex widget tree.

## Core Traits

```rust
pub trait OxideView {
    type Msg;
    type Route: Clone + PartialEq + 'static;

    fn build(&self, ui: &mut Ui<Self::Msg>);
    fn update(&mut self, msg: Self::Msg) -> Option<Self::Route>;

    fn apply_palette(&self, _: &mut Palette) {}
    fn update_frame(&mut self) {}
}
```

Adapter to existing engine:

```rust
pub struct OxideAdapter<V> {
    inner: V,
}

impl<V> View<V::Route> for OxideAdapter<V>
where
    V: OxideView,
{
    fn update(&mut self) {
        self.inner.update_frame();
    }

    fn elements(&self) -> Vec<Element> {
        let mut ui = Ui::new();
        self.inner.build(&mut ui);
        ui.into_elements()
    }

    fn handle_event(&mut self, event: Event) -> Option<V::Route> {
        let mut ui = Ui::new();
        self.inner.build(&mut ui);
        ui.dispatch(&event).and_then(|msg| self.inner.update(msg))
    }

    fn apply_palette(&self, palette: &mut Palette) {
        self.inner.apply_palette(palette);
    }
}
```

## Ui Builder

`Ui<Msg>` owns two outputs:

- `Vec<Element>` for rendering.
- handler registrations for keyboard dispatch.

```rust
pub struct Ui<Msg> {
    elements: Vec<Element>,
    handlers: Vec<KeyHandler<Msg>>,
    modal_depth: usize,
    blocked_by_modal: bool,
}
```

Handlers are registered while building UI. The top modal should take precedence over underlying handlers.

## Message Model

Every screen defines an enum:

```rust
enum ProfilesMsg {
    MoveList(i32),
    OpenSelected,
    DeleteSelected,
    MoveEdit(i32),
    ActivateEdit,
    CommitText(TextField, String),
    CancelText,
    Confirm(QuestionAction),
    CancelDialog,
    Back,
}
```

The `update(msg)` function is the only place that mutates screen state or performs persistence. Rendering should not mutate state.

## Event Dispatch Priority

Order must be deterministic:

1. Top modal handlers.
2. Active editor/focused component handlers.
3. Screen-level handlers.
4. No action.

Underlying screen handlers must be blocked while a modal is active unless explicitly opted in.

## Focus Model

Do not build a complex focus graph initially. Use simple focus scopes:

```rust
pub enum FocusScope {
    Main,
    Editor,
    Modal,
}
```

Handlers carry a scope. Dispatch uses highest active scope.

## Render Order

Render order is append order. Oxide must not reorder elements unless a specific API says it will (for example `modal` draws after the base UI). This is critical for Pascal parity.

## Pascal Parity Rule

Oxide abstractions must never infer new layout behavior by default. Fixed coordinates remain explicit. Helpers should encode existing Pascal patterns, not invent new ones.

Examples:

- `ScreenStyle::MenuSide` maps to Pascal `NewScreen(2,0)`.
- `ModalBox` maps to known two-tone modal box coordinates/colors.
- `FormRow` uses explicit row y/label/value x positions.

## Ownership and Borrowing

Prefer immediate builder methods and plain structs over boxed trait-object trees. Avoid lifetime-heavy retained widget trees.

Good:

```rust
ProfileList::new(self).build(ui);
ui.text(...);
ui.keys().up(Msg::Move(-1));
```

Avoid initially:

```rust
Vec<Box<dyn Widget<Msg> + 'a>>
```

Use traits for reusable components only where they simplify call sites.
