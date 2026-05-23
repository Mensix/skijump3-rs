# Profiles Pilot

Profiles is the validation target for Oxide. It currently demonstrates the pain points Oxide should solve:

- list selection
- edit-mode selection
- text input overlay
- color selector overlay
- numeric replacement selector
- confirmation modal
- persistence after mutations
- palette application for suit/ski colors
- fixed-coordinate profile detail form

## Current Structure

```text
game/src/views/profiles/list.rs
  state, mode enum, event dispatch, view composition

game/src/views/profiles/render.rs
  screen base, list rendering, profile rendering, labels, help text

game/src/views/profiles/actions.rs
  mutations, save triggers, input setup

game/src/views/profiles/format.rs
  profile field value display
```

The split is reasonable given current primitives, but it exists because there is no shared form/list/modal/focus abstraction.

## Target Shape

```rust
enum ProfilesMsg {
    MoveList(i32),
    OpenSelected,
    DeleteSelected,
    MoveEdit(i32),
    ActivateEdit,
    ExitEdit,
    CommitText(TextField, String),
    CancelText,
    CommitColor(ColorField, usize),
    CancelColor(ColorField),
    CommitReplace(usize),
    CancelReplace,
    Confirm(QuestionAction),
    CancelQuestion,
    Back,
}
```

```rust
impl OxideView for ProfilesView {
    type Msg = ProfilesMsg;
    type Route = RouteTarget;

    fn build(&self, ui: &mut Ui<ProfilesMsg>) {
        ui.screen(ScreenStyle::Profiles);
        self.build_profile_list(ui);
        self.build_profile_panel(ui);
        self.build_active_overlay(ui);
    }

    fn update(&mut self, msg: ProfilesMsg) -> Option<RouteTarget> {
        match msg {
            ProfilesMsg::MoveList(delta) => self.move_list(delta),
            ProfilesMsg::OpenSelected => self.open_selected(),
            ProfilesMsg::DeleteSelected => self.delete_selected(),
            ProfilesMsg::Back => Some(RouteTarget::Back),
            _ => None,
        }
    }
}
```

## Build Functions

Initial implementation can keep methods on `ProfilesView` rather than new types:

```rust
impl ProfilesView {
    fn build_profile_list(&self, ui: &mut Ui<ProfilesMsg>) { ... }
    fn build_profile_panel(&self, ui: &mut Ui<ProfilesMsg>) { ... }
    fn build_active_overlay(&self, ui: &mut Ui<ProfilesMsg>) { ... }
}
```

After this is stable, extract reusable `ProfileList` / `ProfilePanel` structs if helpful.

## Event Routing Target

Current `handle_event` has mode-specific branching. Oxide should convert this to declarative bindings:

List mode:

```rust
ui.keys()
    .up(ProfilesMsg::MoveList(-1))
    .down(ProfilesMsg::MoveList(1))
    .enter(ProfilesMsg::OpenSelected)
    .space(ProfilesMsg::OpenSelected)
    .delete(ProfilesMsg::DeleteSelected)
    .backspace(ProfilesMsg::DeleteSelected)
    .escape(ProfilesMsg::Back);
```

Edit mode:

```rust
ui.keys()
    .up(ProfilesMsg::MoveEdit(-1))
    .down(ProfilesMsg::MoveEdit(1))
    .enter(ProfilesMsg::ActivateEdit)
    .space(ProfilesMsg::ActivateEdit)
    .escape(ProfilesMsg::ExitEdit);
```

Modal mode:

```rust
ui.modal(|ui| {
    ConfirmModal::new(...)
        .yes(ProfilesMsg::Confirm(action))
        .no(ProfilesMsg::CancelQuestion)
        .build(ui);
});
```

## Persistence Rule

`build()` must never save or mutate profiles. All save calls remain in `update(msg)` after mutations.

Examples that must still save:

- create new jumper
- add/remove from order
- delete profile
- reset profile
- edit name/real name
- change suit/ski color
- change replacement player
- change coach
- change skip qualification

## Visual Parity Checklist

- Left/right panel colors unchanged.
- Dither fill order unchanged.
- List row y positions unchanged.
- Back row y position unchanged.
- Help text conditional behavior unchanged.
- Profile label/value y positions unchanged.
- Suit/ski swatch coordinates unchanged.
- Edit selection box coordinates unchanged.
- Text input erase box unchanged.
- Confirm modal coordinates unchanged.
- Palette hook behavior unchanged.

## Success Criteria

- `ProfilesView::elements` becomes mostly `build` declarations.
- `ProfilesView::handle_event` becomes `dispatch -> update`.
- `actions.rs` either disappears or becomes small mutation helpers.
- `render.rs` either disappears or becomes profile-specific builder helpers.
- All tests pass.
- Manual profile editing behavior matches current implementation.
