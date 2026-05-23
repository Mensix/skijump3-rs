# Oxide Migration Plan

## Principle

Oxide must be introduced incrementally. Existing views and components must keep compiling during migration. Each migrated screen should be behavior-preserving and tested.

## Phase 0: Baseline

Before implementing Oxide:

- Run `cargo test -p game`.
- Run `cargo build`.
- Note current test count.
- Do not modify physics/jump/competition domain code.

## Phase 1: Foundation

Add `game/src/oxide` with:

- `Ui<Msg>`
- `OxideView`
- `OxideAdapter<V>`
- primitive render methods
- key handler registration
- dispatch ordering tests
- `ScreenStyle`

No production view must migrate in this phase except maybe a tiny private test fixture.

Acceptance:

- Existing tests pass.
- New Oxide unit tests cover handler ordering and modal blocking.

## Phase 2: Tiny Screens

Port screens that have minimal state and existing menu patterns:

1. `WelcomeScreenView`
2. `MainMenuView`
3. `JumpMenuView`

Goals:

- prove `OxideAdapter`
- prove `Menu`/keymap ergonomics
- preserve palette hooks
- preserve route behavior

Acceptance:

- LOC decreases or readability clearly improves.
- `cargo test -p game` passes.
- Manual keyboard behavior matches: Up/Down, Enter/Space, numeric hotkeys, Escape.

## Phase 3: Medium Screens

Port:

1. `TrainingSetupView`
2. `ReplayBrowserView`
3. `CustomCupSetupView`

Goals:

- prove `SelectableList`
- prove panel helpers
- prove page/cycle/clamp navigation
- prove detail panel rendering

Acceptance:

- Exact coordinates/colors preserved.
- Custom Cup key behavior unchanged.
- Replay browser still loads save+asset replays and strips `.SJR` case-insensitively.

## Phase 4: Components

Migrate or wrap existing components:

- `Menu`
- `TextInput`
- `ValueSelector`
- `ConfirmDialog`
- `SaveReplayDialog`

Do this only after tiny/medium views prove the API. Avoid prematurely rewriting every component.

Acceptance:

- Component APIs emit typed messages.
- Existing call sites can be migrated one at a time.
- No regression to dialog/modal behavior.

## Phase 5: Profiles Pilot

Port Profiles after the primitives are proven.

Goals:

- replace scattered render/action/event code with `build/update`
- use `Form`, `SelectableList`, `TextField`, `ValueField`, `ConfirmModal`
- keep persistence in `update`, never in `build`

Acceptance:

- All profile mutations still save `PLAYERS.SKI`.
- All edit modes behave the same.
- Confirm dialogs still block underlying handlers.
- Palette suit/ski previews remain correct.

## Phase 6: Records and Remaining UI Screens

Port records browsing after pager/list abstractions stabilize.

Leave gameplay-heavy views (`TrainingJumpView`, `WorldCupJumpView`) mostly untouched unless Oxide helps overlays. These screens are simulation-driven, not form-driven.

## Phase 7: Cleanup

After enough migrations:

- remove unused old component helpers
- decide whether `engine::ui::Component` remains or is deprecated
- document final Oxide style in `docs/oxide`
- update examples

## Commit Strategy

Use small commits:

1. foundation
2. one widget/component
3. one screen migration
4. tests/fixes

Do not combine framework work and unrelated gameplay changes.
