# Oxide UI Architecture

Oxide is the proposed game-level UI authoring layer for Ski Jump 3 Rust. It sits above `engine::ui::Element` and below game views. The goal is to make UI screens declarative, typed, keyboard-first, and Pascal-parity-safe without introducing a web-style runtime framework.

## Documents

- `architecture.md` - core thesis, boundaries, runtime model, and non-goals.
- `api.md` - proposed Rust API surface and code sketches.
- `components.md` - widgets/components that Oxide should provide.
- `migration.md` - incremental migration plan and view order.
- `todo.md` - executable implementation checklist for DeepSeek/agents.
- `profiles-pilot.md` - how to use Profiles as the validation screen.
- `testing.md` - tests, parity checks, and acceptance criteria.
- `deepseek-brief.md` - direct one-shot implementation brief for an agent.

## Executive Summary

The current UI engine has good render primitives and weak authoring primitives. Every non-trivial view manually coordinates:

- render order
- keyboard dispatch
- focus/mode state
- overlays/modals
- selection wrapping
- form rows
- Pascal screen templates
- persistence side effects

Oxide should solve this with an immediate-mode, typed-message UI toolkit:

```text
state -> build UI -> dispatch event -> Msg -> update state -> rerender
```

Keep `Element` as the final rendering primitive. Do not implement a virtual DOM, diffing, async lifecycle, or general-purpose frontend framework.

## Placement

Add Oxide under:

```text
game/src/oxide/
```

Do not move it into `engine` initially. Oxide is game-specific and may depend on Pascal UI conventions, 320x200 coordinates, palette constants, `LangBase`, and existing game components.

## First Implementation Rule

Oxide must be additive. Existing `View<RouteTarget>` implementations keep working while screens migrate one by one through an adapter.
