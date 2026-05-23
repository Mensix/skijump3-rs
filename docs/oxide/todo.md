# Oxide Implementation TODO

This is an executable checklist for DeepSeek/agents.

## Foundation

- [ ] Create `game/src/oxide/mod.rs`.
- [ ] Create `game/src/oxide/ui.rs`.
- [ ] Create `game/src/oxide/adapter.rs`.
- [ ] Create `game/src/oxide/handler.rs`.
- [ ] Create `game/src/oxide/screen.rs`.
- [ ] Export `pub mod oxide;` from `game/src/lib.rs` or relevant module root if needed.
- [ ] Define `OxideView` trait.
- [ ] Define `Ui<Msg>` with `elements`, `handlers`, modal state.
- [ ] Implement `Ui::new()` and `Ui::into_elements()`.
- [ ] Implement primitive render methods: `push`, `extend`, `text`, `right_text`, `center_text`, `fillbox`, `box_`, `fill_area`, `sprite`.
- [ ] Implement `Ui::on_key` and `Ui::on_keys`.
- [ ] Implement `Ui::dispatch`.
- [ ] Implement `OxideAdapter<V>` for `engine::ui::View`.
- [ ] Add dispatch-order unit tests.
- [ ] Add modal-blocking unit tests.

## Screen Templates

- [ ] Define `ScreenStyle` enum.
- [ ] Implement `Ui::screen(ScreenStyle)`.
- [ ] Map `ScreenStyle::MenuTop` to existing `new_screen(1)`.
- [ ] Map `ScreenStyle::MenuSide` to existing `new_screen(2)`.
- [ ] Map `ScreenStyle::SplitRecords` to existing `new_screen(4)`.
- [ ] Map `ScreenStyle::FullPanel` to existing `new_screen(5)`.
- [ ] Add `ScreenStyle::Profiles` matching `profiles/render.rs::draw_screen_base`.
- [ ] Add comments with Pascal `NewScreen` references.

## KeyMap

- [ ] Create `game/src/oxide/keymap.rs`.
- [ ] Implement `Ui::keys()`.
- [ ] Implement builder methods: `up`, `down`, `left`, `right`, `enter`, `space`, `escape`, `delete`, `backspace`, `home`, `end`, `page_up`, `page_down`, `char`.
- [ ] Add tests for multi-key same-message mappings.

## Menu

- [ ] Create `game/src/oxide/menu.rs`.
- [ ] Implement menu builder with explicit x/y/item_w/item_h.
- [ ] Support labels, selection box, exit row, y offsets.
- [ ] Support numeric hotkeys.
- [ ] Support Up/Down wrapping.
- [ ] Support Enter/Space/Escape.
- [ ] Port `MainMenuView`.
- [ ] Port `JumpMenuView`.
- [ ] Port `WelcomeScreenView` if menu API fits.

## SelectableList

- [ ] Create `game/src/oxide/list.rs`.
- [ ] Implement selected index, wrap/clamp mode, custom row rendering.
- [ ] Implement movement messages using `i32` delta or explicit actions.
- [ ] Support selection box renderer.
- [ ] Port `ReplayBrowserView` selected replay navigation.
- [ ] Port `TrainingSetupView` hill list if API reduces code.
- [ ] Port `CustomCupSetupView` selected hill list only if exact behavior stays clear.

## Modal

- [ ] Create `game/src/oxide/modal.rs`.
- [ ] Implement `Ui::modal` that blocks lower-scope handlers.
- [ ] Implement `Ui::overlay` that does not block handlers.
- [ ] Implement `modal_box` helper using current `modal_background` colors.
- [ ] Implement `ConfirmModal`.
- [ ] Add tests: modal Escape wins over main Escape.

## Text and Value Editors

- [ ] Create `game/src/oxide/text_field.rs` or place in `component.rs`.
- [ ] Port `TextInput` logic to typed Oxide messages.
- [ ] Create `ValueField` from `ValueSelector` logic.
- [ ] Preserve wrap/clamp behavior.
- [ ] Preserve cursor blink behavior.
- [ ] Preserve exact erase boxes and cursor coordinates.

## Form

- [ ] Create `game/src/oxide/form.rs`.
- [ ] Implement explicit-coordinate form rows.
- [ ] Support dynamic value x based on label width.
- [ ] Support selected row highlight.
- [ ] Support read-only rows.
- [ ] Support custom row drawing callback/function.
- [ ] Use in Profiles pilot.

## Profiles Pilot

- [ ] Create `ProfilesMsg` enum.
- [ ] Implement `OxideView for ProfilesView` behind adapter or create `ProfilesOxideView`.
- [ ] Move list keyboard handling to messages.
- [ ] Move edit keyboard handling to messages.
- [ ] Keep all persistence in `update(msg)`.
- [ ] Keep format helpers (`format_profile_value`) if useful.
- [ ] Replace `draw_screen_base`, `draw_list`, `draw_profile` with Oxide build functions/components.
- [ ] Preserve `ConfirmDialog` behavior through `ConfirmModal` or adapter.
- [ ] Verify all mutations still call `save_players`.

## Verification Commands

- [ ] Run `cargo test -p game` after every phase.
- [ ] Run `cargo build` after every migrated screen.
- [ ] Manually smoke-test keyboard controls for migrated screens.

## Do Not Do

- [ ] Do not rewrite jump physics or competition domain.
- [ ] Do not introduce virtual DOM/diffing.
- [ ] Do not move Oxide into `engine` until at least several screens are migrated.
- [ ] Do not add macros before the API is proven.
- [ ] Do not auto-layout Pascal screens unless a row/list pattern already exists.
