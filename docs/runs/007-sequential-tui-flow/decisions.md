# Decisions — 007 Sequential TUI Flow

## Task 1: app.rs state machine

- **Panel enum removed entirely.** The spec says it "may be deleted" — with no panel-cycling in the new design, there is no reason to keep it. `focused_panel` field also removed.
- **`toggle_show_complete()` / `toggle_show_blocked()` added as direct methods.** The spec says `h`/`c`/`b` keybinds should all call `self.prefs.save()` — mirroring the existing `toggle_headless()` pattern.
- **`toggle_option()` and `OPTIONS_ITEMS` deleted.** Only three callers existed; all are replaced by direct `toggle_headless`, `toggle_show_complete`, `toggle_show_blocked` methods.
- **`next_panel()` deleted.** With Panel enum removed, Tab switching is handled exclusively by `switch_tab()` directly from the event loop.
- **`options_index` field deleted.** No Options panel, no index needed.
- **`move_up` / `move_down` simplified.** Panel dispatch removed; navigation always operates on the active spec tab (or delegates to metrics scroll). Team popup navigation still uses `popup_move_up/down`.
- **`popup_move_up/down` extended to handle `TeamDialog`.** The `selected_index` inside `TeamDialog` is the only popup navigation state for teams.
- **`dismiss_popup()` on `ActionDialog` restores `TeamDialog`.** Esc re-opens the team popup at `self.team_index`, matching spec requirement.

## Task 2: ui.rs layout and rendering

- **Unit tests not written for ui.rs rendering.** The render functions produce terminal output and cannot be meaningfully unit-tested without a backend mock. The event loop wiring (h/c/b keys, Tab → switch_tab) is implicitly exercised by the integration smoke test in Task 3. Logged here to document the TDD deviation.
- **`KeyCode::Char('h')` / `'c'` / `'b'` wired to new toggle methods.** Previous `'h'` called `toggle_headless()` (unchanged); `'c'` and `'b'` are new handlers.
- **Tab key changed from `app.next_panel()` to `app.switch_tab()`.** Panel cycling is gone; Tab now only switches spec/requirements tabs.
- **Footer updated to show pref state inline.** Format: `h:headless[on/off]  c:complete[on/off]  b:blocked[on/off]  Tab switch tab  Enter select  q quit`.
