# Oxide Testing and Acceptance

## Unit Tests

Add tests for Oxide primitives before migrating real screens.

Required tests:

- `Ui::dispatch` returns matching key handler.
- Later handlers in same scope override earlier handlers if needed.
- Modal handlers block main handlers.
- Editor handlers beat main handlers when no modal exists.
- `on_keys` maps multiple keys to the same message.
- `ScreenStyle` emits expected element counts or exact element sequences for stable templates.
- `SelectableList` movement wraps/clamps as configured.
- `Form` selected row box coordinates are stable.

## Golden Element Tests

For migrated screens, consider element-level snapshot tests where practical. Do not snapshot huge dynamic views unless stable. Useful targets:

- Main menu static elements.
- Jump menu static elements.
- Welcome screen static elements for a fixed language list.
- Modal background element sequence.
- Screen templates.

Element snapshots should compare `Debug` output or exact `Element` vectors where values are stable.

## Behavior Tests

For each migrated screen, test event-to-message/update behavior:

- Up/Down changes selection correctly.
- Enter/Space submits.
- Escape routes back or cancels overlay.
- Modal blocks underlying Escape/Enter handlers.
- Numeric hotkeys select menu items.

## Manual Smoke Tests

After each migrated screen:

- Launch game.
- Navigate to migrated screen.
- Exercise all documented keys.
- Verify visual layout against previous behavior.
- Verify palette hooks still apply.

Profiles manual smoke:

- Create jumper.
- Rename jumper.
- Edit real name.
- Change suit and ski colors.
- Set replacement player.
- Change coach style.
- Change skip qualification.
- Add/remove from order.
- Delete profile via confirmation.
- Reset profile via confirmation.
- Return to main menu.
- Restart and verify profile persistence.

Replay browser manual smoke:

- Asset replay appears.
- Save replay appears.
- Lowercase `.sjr` display stem is stripped.
- Broken replay shows error state.
- Left/right/up/down cycle correctly.
- Enter starts playback.
- Escape returns.

Custom Cup manual smoke:

- Left/right clamps preview.
- PageUp/PageDown ±5 clamps.
- Home/Delete reset preview.
- End jumps to last hill.
- Down/Space adds hill.
- Up/Backspace removes hill.
- Enter starts cup.
- Escape returns.

## Verification Commands

Run after every meaningful migration:

```bash
cargo test -p game
cargo build
```

## Acceptance Criteria

A migration is acceptable only if:

- tests pass
- build passes
- visible coordinates/colors are unchanged unless explicitly intended
- key behavior is unchanged
- persistence behavior is unchanged
- code is simpler to read than before

## Regression Risks

Watch for:

- modal overlays hiding underlying form by accident
- handlers firing under modals
- lost Escape behavior
- lost numeric hotkeys
- changed render order due to helper extraction
- silently changed row y positions
- accidental state mutation inside `build`
- borrow conflicts caused by long-lived references in builders
