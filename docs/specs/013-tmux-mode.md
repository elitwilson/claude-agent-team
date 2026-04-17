---
number: 013
status: ready
base_branch: main
---

# Feature: Tmux Mode

## Summary

Add a tmux mode that spawns each agent run in its own tmux window or pane instead of taking over the current terminal. When active, `claude` processes are launched via `tmux new-window` or `tmux split-window` and the caller returns immediately (spawn and forget). Layout is configurable via prefs. Tmux mode activates automatically when `$TMUX` is set, or can be forced explicitly with `--tmux`.

---

## Requirements

- When `$TMUX` is set in the environment, tmux mode activates automatically with no user action required
- `--tmux` flag forces tmux mode regardless of environment
- If `--tmux` is passed but `$TMUX` is not set, warn to stderr and fall back to existing interactive behavior
- Tmux layout is configurable in prefs (`tmux_layout = "window"` or `"pane"`)
- Default layout is `window` — each agent run gets a new named tmux window
- `pane` layout splits the current window instead
- Window/pane name is the spec slug
- In tmux mode, the caller spawns and returns immediately (no waiting for the process)
- Tmux mode is incompatible with headless mode — if both are active, headless takes priority and tmux is silently skipped

---

## Scope

### In Scope

- `TmuxLayout` enum and `tmux_layout` field in `Prefs`
- `--tmux` flag added to `RunArgs`
- `run_claude_in_tmux()` in `runner.rs`
- Env var detection and flag wiring in `main.rs`
- Graceful fallback when `--tmux` is passed outside a tmux session

### Out of Scope

- Tracking which pane/window each agent is running in
- TUI integration (no pane switcher or status indicators)
- Named tmux sessions or session management
- Multi-agent coordination within a single window (each agent independently placed)

---

## Technical Approach

### `TmuxLayout` enum and prefs field

Add to `src/prefs.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TmuxLayout {
    #[default]
    Window,
    Pane,
}

// In Prefs struct:
#[serde(default)]
pub tmux_layout: TmuxLayout,
```

Serializes as `tmux_layout = "window"` or `tmux_layout = "pane"` in the TOML prefs file.

### `--tmux` flag in `RunArgs`

Add to `src/run_cmd.rs`:

```rust
pub struct RunArgs {
    // ... existing fields ...
    pub tmux: bool,
}
```

Parse `"--tmux"` in `parse_run_args` setting `tmux = true`.

### `run_claude_in_tmux()` in `runner.rs`

```rust
pub fn run_claude_in_tmux(
    rendered_prompt: &str,
    spec_slug: &str,
    layout: TmuxLayout,
    oauth_token: Option<&str>,
) -> Result<()>
```

Builds the claude arg string, then dispatches:

- `TmuxLayout::Window`: `tmux new-window -n "<spec-slug>" "claude <args>"`
- `TmuxLayout::Pane`: `tmux split-window -d "claude <args>"`

Sets `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` via shell env prefix in the command string, and `CLAUDE_CODE_OAUTH_TOKEN` if provided.

### Activation logic in `main.rs`

```rust
let tmux_available = std::env::var("TMUX").is_ok();
let use_tmux = run_args.tmux || tmux_available;

if run_args.tmux && !tmux_available {
    eprintln!("Warning: --tmux passed but $TMUX is not set — falling back to interactive mode");
}

if use_tmux && !run_args.headless {
    run_claude_in_tmux(&prompt, &spec_slug, prefs.tmux_layout, oauth_token.as_deref())?;
} else {
    run_claude(&prompt, run_args.headless, &log_path, oauth_token.as_deref())?;
}
```

---

## Success Criteria

- [ ] `tmux_layout = "window"` in prefs causes `tmux new-window` to be called with the spec slug as the window name
- [ ] `tmux_layout = "pane"` causes `tmux split-window` to be called
- [ ] `$TMUX` set → tmux mode activates without any flags
- [ ] `--tmux` flag → tmux mode activates regardless of `$TMUX`
- [ ] `--tmux` without `$TMUX` set → warning printed, falls back to interactive
- [ ] `--headless` takes priority over tmux mode when both are active
- [ ] Default prefs (no `tmux_layout` key) resolve to `window` layout

---

## Tasks

- [ ] **`src/prefs.rs` — `TmuxLayout` enum and field:** Add `TmuxLayout` enum with `Window` and `Pane` variants. Add `tmux_layout: TmuxLayout` to `Prefs` with `#[serde(default)]`. Update `Default` impl. Add unit tests covering TOML round-trip for both variants and missing key defaulting to `window`.

- [ ] **`src/run_cmd.rs` — `--tmux` flag:** Add `tmux: bool` to `RunArgs`. Parse `"--tmux"` in `parse_run_args`. Update existing tests and add a test for the new flag.

- [ ] **`src/runner.rs` — `run_claude_in_tmux()`:** Implement the function. Build the full claude command string with env vars prefixed, dispatch to `tmux new-window` or `tmux split-window` based on layout. Unit-test the command string construction without actually calling tmux (test the arg-building logic separately).

- [ ] **`main.rs` — activation wiring:** Read `$TMUX`, check `run_args.tmux`, emit warning on mismatch, dispatch to `run_claude_in_tmux` vs `run_claude` based on resolved mode.

---

## Considerations

- Env vars (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`, `CLAUDE_CODE_OAUTH_TOKEN`) must be passed into the tmux-spawned shell. Since tmux inherits the parent environment by default, the token env var approach from `run_claude` should work the same way — set it on the `Command` that invokes `tmux`.
- The claude args string passed to `tmux new-window` must be properly shell-escaped to survive the tmux command parsing. Use `shlex`-style quoting or build the args list carefully.
- `tmux split-window -d` suppresses focus stealing so the user's current pane stays active. This is intentional.
- Headless-over-tmux priority is a deliberate escape hatch for scheduled runs that happen to run inside a tmux session — they should never accidentally open interactive windows.
