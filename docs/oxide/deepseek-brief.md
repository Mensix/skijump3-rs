# DeepSeek One-Shot Brief: Oxide UI Layer

## Mission

Implement Oxide, an additive game-level UI authoring layer for Ski Jump 3 Rust. Oxide should make UI screens declarative, typed-message-driven, keyboard-first, and Pascal-parity-safe while keeping the existing `engine::ui::Element` renderer intact.

Read all docs in `docs/oxide/` before coding.

## Required Reading Order

1. `README.md`
2. `architecture.md`
3. `api.md`
4. `components.md`
5. `migration.md`
6. `todo.md`
7. `profiles-pilot.md`
8. `testing.md`

## Hard Constraints

- Do not rewrite physics, jump simulation, competition domain, persistence format, or renderer.
- Do not move Oxide into `engine` initially.
- Do not introduce virtual DOM, diffing, CSS/flexbox, async lifecycle, or macros.
- Do not change visible coordinates/colors/render order unless explicitly required.
- Do not mutate state inside `build()`.
- Keep all persistence and side effects inside `update(msg)` or existing mutation helpers.
- Every migration must pass `cargo test -p game` and `cargo build`.

## Target Architecture

Create:

```text
game/src/oxide/
  mod.rs
  adapter.rs
  ui.rs
  handler.rs
  screen.rs
  keymap.rs
  menu.rs
  list.rs
  form.rs
  modal.rs
  component.rs
```

Minimum initial traits/types:

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

```rust
pub struct Ui<Msg> {
    elements: Vec<Element>,
    handlers: Vec<KeyHandler<Msg>>,
    modal_depth: usize,
}
```

```rust
pub struct OxideAdapter<V> {
    inner: V,
}
```

The adapter must implement existing `engine::ui::View` so routes can migrate one at a time.

## First Deliverable

Do not start by porting Profiles.

First deliver:

- Oxide foundation modules.
- `Ui` primitive render methods.
- key handler registration/dispatch.
- modal blocking.
- `ScreenStyle` templates.
- unit tests for dispatch and modal behavior.
- one tiny migrated screen (`MainMenuView` or `JumpMenuView`) through `OxideAdapter`.

## Migration Order

Use this order unless there is a concrete blocker:

1. Foundation only.
2. `MainMenuView`.
3. `JumpMenuView`.
4. `WelcomeScreenView`.
5. `TrainingSetupView`.
6. `ReplayBrowserView`.
7. `CustomCupSetupView`.
8. Reusable components (`ConfirmDialog`, `TextInput`, `ValueSelector`, `SaveReplayDialog`) as needed.
9. `ProfilesView` pilot.
10. Records views.

Leave gameplay-heavy jump/competition views mostly alone until the framework proves itself.

## API Priorities

Implement APIs in this priority order:

1. `Ui` primitives and `OxideView`.
2. `KeyMap` and handler dispatch.
3. `ScreenStyle`.
4. `Menu` builder.
5. `Modal` / `ConfirmModal`.
6. `SelectableList`.
7. `TextField` / value editors.
8. `Form`.

## Quality Bar

Good Oxide code should look like this:

```rust
impl OxideView for MainMenuView {
    type Msg = MainMenuMsg;
    type Route = RouteTarget;

    fn build(&self, ui: &mut Ui<MainMenuMsg>) {
        ui.extend(self.layout.background());
        ui.menu()
            .at(11, 97)
            .selected(self.menu.selected())
            .item(1, self.layout.langbase.lstr(20), MainMenuMsg::Choose(1))
            .item(2, self.layout.langbase.lstr(21), MainMenuMsg::Choose(2))
            .exit(self.layout.langbase.lstr(26), MainMenuMsg::Back)
            .build();
        ui.extend(self.layout.footer());
    }

    fn update(&mut self, msg: MainMenuMsg) -> Option<RouteTarget> {
        match msg {
            MainMenuMsg::Choose(n) => self.choose(n),
            MainMenuMsg::Back => Some(RouteTarget::Back),
        }
    }
}
```

Do not force this exact syntax if Rust ergonomics argue otherwise, but keep the same separation of concerns.

## Acceptance Criteria For Each Ported Screen

- The migrated screen compiles through `OxideAdapter`.
- Old route behavior is preserved.
- Palette hooks are preserved.
- Keyboard behavior is preserved.
- Exact visual element order is preserved unless explicitly documented.
- The new code is shorter or materially clearer.
- Tests pass.

## Red Flags

Stop and reassess if:

- lifetimes become complicated due to retained widget trees
- every widget needs `Box<dyn Widget>`
- `build()` needs mutable access to view state
- event dispatch requires hidden global state
- a screen migration increases LOC substantially without readability gains
- Pascal parity becomes harder to reason about

## Commit Strategy

Commit frequently:

1. foundation modules + tests
2. keymap/modal tests
3. first menu migration
4. each migrated screen
5. each component migration

Each commit should be independently buildable.
