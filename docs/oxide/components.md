# Oxide Components

This document defines the initial component set. Build these only as screens need them. Avoid speculative widgets.

## Primitive Rendering Helpers

These are thin wrappers around `Element`:

- `text`
- `right_text`
- `center_text`
- `fillbox`
- `box_`
- `fill_area`
- `sprite`
- `image`
- `extend`

They should append elements in exact call order.

## Screen Templates

`ScreenStyle` replaces numeric `new_screen(style)` at authoring sites.

Required variants:

- `Black`
- `MenuTop`
- `MenuSide`
- `SplitRecords`
- `FullPanel`
- `Profiles`

The implementation may delegate to existing `game/src/components/screen.rs` initially.

## KeyMap

Reusable key registration builder.

Must support:

- arrows
- Home/End/PageUp/PageDown
- Enter/Space/Escape/Delete/Backspace
- specific char keys
- multiple keys mapped to the same message

## Menu

Replacement/wrapper for `game/src/components/menu.rs`.

Responsibilities:

- render numbered items
- render optional exit item
- render selection box
- handle Up/Down wrapping
- handle Enter/Space
- handle numeric shortcuts
- handle Escape

It should support existing exact coordinates and item y offsets.

## SelectableList

General list primitive for Profiles, replay browser, records pagination, and training hill list.

Responsibilities:

- selection index
- wrap or clamp mode
- row count
- custom row rendering
- selection box rendering
- event bindings for Up/Down/Left/Right/Page keys where configured

Do not force a visual style. Provide hooks/builders for row renderers.

## Pager

Primitive for screen pages.

Responsibilities:

- current page
- page count
- Back/First/Prev/Next action mapping
- page hint rendering through existing `page_hints`

This should replace local `PageAction` implementations after records views migrate.

## Form

Profiles proves that forms are first-class.

Responsibilities:

- selectable rows
- label/value pairs
- dynamic value x from label width
- row y calculation
- read-only rows
- custom row renderer
- highlight box
- event bindings for Up/Down/Enter/Escape

## TextField

Oxide wrapper around `TextEditState` and cursor blink.

Responsibilities:

- render background erase box
- render buffer
- render cursor
- handle Backspace/Delete/Char/Enter/Escape
- emit typed `Commit(String)` / `Cancel` messages

This can start by adapting current `TextInput` logic.

## ValueField

Oxide replacement for `ValueSelector`.

Modes:

- color bars
- numeric value
- named replacement player

Responsibilities:

- wrap/clamp navigation
- PageUp/PageDown increments
- Home/End
- Enter/Space commit
- Escape cancel

## ConfirmModal

Wrapper for confirmation dialogs.

Responsibilities:

- modal background
- message text
- yes/no prompt
- blinking cursor
- Y/N/Escape handlers

Use exact existing coordinates unless parameters are supplied.

## Overlay

Some screens have overlays that are not blocking modals (competition info panels). Oxide should support render-only overlays separately from blocking modals.

Initial API:

```rust
ui.overlay(|ui| { ... }); // render after main, does not block handlers
ui.modal(|ui| { ... });   // render after main, blocks handlers
```

## MainLayout Integration

Existing `MainLayout` can stay. Oxide should provide helpers that append `layout.background_element()`, `header_elements`, menu, and footer for main menu screens.

Do not force all screens to use `MainLayout`.
