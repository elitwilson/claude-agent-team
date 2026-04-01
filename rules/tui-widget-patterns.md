---
version: 0.1.0
updated: 2026-04-01
paths:
  - "src/tui/**/*.rs"
---

# TUI Widget Patterns

## Component Philosophy

Ratatui widgets are the component primitive for this codebase. The mental model maps directly to component-based frontend development:

| Frontend (Vue) | Ratatui equivalent |
|---|---|
| `.vue` file | `struct` + `impl Widget` |
| Props | Struct fields |
| Template | `Widget::render` body |
| `components/` directory | `src/tui/widgets.rs` |

---

## !! CRITICAL !! Widget Over Free Functions

**Reusable UI must be expressed as `impl Widget`, not as `render_*` free functions.**

```rust
// ❌ Avoid
fn render_team_dialog(f: &mut Frame, teams: &[String], selected: usize) { ... }

// ✅ Correct
pub struct TeamDialog<'a> {
    pub teams: &'a [String],
    pub selected_index: usize,
}

impl Widget for TeamDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) { ... }
}
```

Free functions are acceptable only for private helpers that are not UI components (e.g., `popup_frame`).

---

## !! CRITICAL !! No State Mutation in Render

**`Widget::render` must be a pure read of its inputs. It must not mutate any state.**

```rust
// ❌ Avoid — render setting state for the event loop to read
fn render(self, area: Rect, buf: &mut Buffer) {
    self.state.visible_rows = area.height as usize; // mutation in render
}

// ✅ Correct — render reads only
fn render(self, area: Rect, buf: &mut Buffer) {
    let visible_rows = area.height as usize; // local, not stored
}
```

If a value can only be known at render time (e.g., terminal height), compute it locally and do not write it back to `App` or any shared state.

---

## Narrow Inputs

Widget structs should declare only the fields they actually need. Do not pass `&App` into a widget.

```rust
// ❌ Avoid — widget knows too much
pub struct TeamDialog<'a> {
    pub app: &'a App,
}

// ✅ Correct — explicit, minimal props
pub struct TeamDialog<'a> {
    pub teams: &'a [String],
    pub selected_index: usize,
}
```

---

## Where Widgets Live

Custom widgets belong in `src/tui/widgets.rs`. Shared private helpers (e.g., `popup_frame`) live in the same file, not exported.

Screens with substantial standalone state (e.g., `metrics.rs`, `schedule_picker.rs`) keep their own render function — these are screen-level, not reusable components.

---

## Popup Routing

Popup overlays are rendered via a `match` on `app.popup` at the end of the top-level `render()`. Do not use `if let` chains with `return` statements.

```rust
// ✅ Correct
match &app.popup {
    Some(PopupAction::TeamDialog { selected_index }) =>
        f.render_widget(TeamDialog { teams: &app.teams, selected_index: *selected_index }, f.area()),
    // ...
    None => {}
}
```
