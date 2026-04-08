---
number: 011
status: complete
base_branch: main
---

# Feature: Custom Teams

## Summary

Users can define their own teams and agents outside the built-in set embedded in the binary. Custom teams can be defined at two levels: user-level (global across all projects, in `~/.claude-launch/user/`) or project-level (local to a specific project, configured via `.claude-launch.toml`). All custom teams appear alongside built-in teams in the TUI. Name collisions across any two sources are rejected at startup with a clear error.

---

## Requirements

- A user can place a team `.md` file in `~/.claude-launch/user/teams/` and see it appear in the TUI alongside built-in teams
- A user can place agent `.md` files in `~/.claude-launch/user/agents/<team-name>/` and reference them from their user-level team prompt via `${USER_DIR}`
- A user can set `custom_dir = "path/to/dir"` in `.claude-launch.toml` (relative to project root) to define project-level teams
- Project-level teams follow the same directory convention: `<custom_dir>/teams/` and `<custom_dir>/agents/<team-name>/`
- Project-level team prompts can reference their agents via `${PROJECT_DIR}`
- If any team name appears in more than one source (built-in, user, project), the TUI fails to launch with an error that names the conflicting team(s)
- `~/.claude-launch/user/teams/` and `~/.claude-launch/user/agents/` are created on first install
- The binary never writes to or overwrites anything inside `~/.claude-launch/user/`
- If `custom_dir` is configured but the directory does not exist on disk, the binary fails fast with a clear error
- `${USER_DIR}` and `${PROJECT_DIR}` are substituted in all team prompts at render time, the same way `${WORKFLOW_DIR}` is today

---

## Scope

### In Scope

- `TeamEntry` struct and `TeamSource` enum for tracking which source a team came from
- Updated `discover_teams()` that merges built-in, user-level, and project-level sources with collision detection
- `custom_dir` config field in `Config`
- `${USER_DIR}` and `${PROJECT_DIR}` substitutions in `render_prompt()`
- Creation of `~/.claude-launch/user/teams/` and `~/.claude-launch/user/agents/` on install and on `resolve_workflow_dir()`
- Wiring in `main.rs` for both interactive and scheduled run paths

### Out of Scope

- TUI scaffolding for creating new custom teams (future idea, not this spec)
- Grouping or labeling teams by source in the TUI (all teams shown in one flat sorted list)
- Custom drafter agents (drafter path remains hardcoded)
- `${USER_DIR}` or `${PROJECT_DIR}` substitution in `render_drafter_prompt()` (drafter is built-in and does not reference custom agent files)

---

## Technical Approach

### Data model

Add to `src/config.rs`:

```rust
pub enum TeamSource {
    BuiltIn,
    User,
    Project,
}

pub struct TeamEntry {
    pub name: String,
    pub path: PathBuf,   // absolute path to the .md file
    pub source: TeamSource,
}
```

### `discover_teams()` signature change

Current signature:
```rust
pub fn discover_teams(teams_dir: &Path) -> Result<Vec<String>>
```

New signature:
```rust
pub fn discover_teams(
    builtin_teams_dir: &Path,
    user_teams_dir: &Path,
    project_teams_dir: Option<&Path>,
) -> Result<Vec<TeamEntry>>
```

Behavior:
- Read `.md` files from each directory that exists (missing user or project dirs are silently skipped — no entries, no error)
- If `project_teams_dir` is `Some` but the path does not exist on disk, return an error (configured but missing = fail fast)
- Collect all names across all sources; if any name appears more than once, return an error listing every conflicting name
- Return entries sorted by name

### `Config` change

Add to the `Config` struct in `src/config.rs`:

```rust
#[serde(default)]
pub custom_dir: Option<String>,
```

Resolved at runtime relative to the project root (same convention as `specs_dir`).

### `render_prompt()` change

Add two parameters to `render_prompt()` in `src/prompt.rs`:

```rust
pub fn render_prompt(
    template_path: &Path,
    spec_file: &str,
    feature_slug: &str,
    workflow_dir: &str,
    team: &str,
    user_dir: &str,        // new
    project_dir: &str,     // new — empty string if custom_dir not configured
) -> Result<String>
```

Add substitutions:
```rust
.replace("${USER_DIR}", user_dir)
.replace("${PROJECT_DIR}", project_dir)
```

`${PROJECT_DIR}` substitutes to an empty string when `custom_dir` is not configured. Built-in and user-level team prompts never reference `${PROJECT_DIR}`, so the empty substitution is harmless.

### `resolve_workflow_dir()` and install changes

In `src/prompt.rs`, after extracting embedded files, also create:
```
~/.claude-launch/user/teams/
~/.claude-launch/user/agents/
```

Use `create_dir_all` — idempotent, safe to call on every run.

In `src/install.rs` `run_install()`, also create the same two directories explicitly so they exist after first install regardless of whether a run has happened yet.

### Template path change in `main.rs`

Currently the template path is constructed from `workflow_dir` + team name:
```rust
let template_path = Path::new(&workflow_dir)
    .join("prompts").join("teams")
    .join(format!("{}.md", selection.team));
```

After this change, use `TeamEntry.path` directly — it already holds the correct absolute path regardless of source. Look up the selected team entry by name from the discovered list.

### `main.rs` wiring

Compute these values early in both `run()` and `run_scheduled()`:

```rust
let user_dir = format!("{}/user", workflow_dir);
let project_dir = config.custom_dir
    .as_ref()
    .map(|d| cwd.join(d).to_string_lossy().into_owned())
    .unwrap_or_default();
let project_teams_dir = config.custom_dir
    .as_ref()
    .map(|d| cwd.join(d).join("teams"));
```

Pass `user_teams_dir` and `project_teams_dir` to `discover_teams()`. Pass `user_dir` and `project_dir` to `render_prompt()`.

The TUI call site passes team names extracted from `Vec<TeamEntry>` — the existing `Vec<String>` interface to the TUI does not need to change. After selection, find the matching `TeamEntry` by name to get the path.

---

## Success Criteria

- [ ] A `.md` file placed in `~/.claude-launch/user/teams/` appears in the TUI
- [ ] A user-level team prompt containing `${USER_DIR}/agents/foo/coder.md` renders with the correct resolved path
- [ ] `custom_dir = "custom-teams"` in `.claude-launch.toml` causes `custom-teams/teams/*.md` files to appear in the TUI
- [ ] A project-level team prompt containing `${PROJECT_DIR}/agents/foo/coder.md` renders with the correct resolved path
- [ ] TUI fails to launch with a named error when any team name exists in more than one source
- [ ] `~/.claude-launch/user/teams/` and `~/.claude-launch/user/agents/` exist after install
- [ ] The binary never modifies files in `~/.claude-launch/user/`
- [ ] `custom_dir` pointing to a non-existent directory produces a clear error at startup, not a panic

---

## Tasks

- [ ] **`TeamEntry` + `discover_teams()` rewrite:** Add `TeamSource` enum and `TeamEntry` struct to `src/config.rs`. Rewrite `discover_teams()` with the new three-source signature, collision detection, and graceful handling of missing dirs. Unit-test all cases: missing user dir, missing project dir, configured-but-missing project dir (error), collisions across all source pairs, clean merge. All tests must pass before proceeding.

- [ ] **`Config` `custom_dir` field:** Add `custom_dir: Option<String>` to `Config` in `src/config.rs`. Add tests for TOML parsing with and without the field. Depends on Task 1 (shares the file).

- [ ] **`render_prompt()` + user dir creation:** Add `user_dir` and `project_dir` params to `render_prompt()` in `src/prompt.rs` with corresponding substitutions. In `resolve_workflow_dir()`, create `~/.claude-launch/user/teams/` and `~/.claude-launch/user/agents/` via `create_dir_all`. In `src/install.rs` `run_install()`, do the same. Update all existing call sites to pass the new params (use empty strings as placeholders until Task 4 wires real values). Unit-test the substitutions.

- [ ] **`main.rs` wiring:** Compute `user_dir`, `project_dir`, and `project_teams_dir` from config and workflow_dir. Pass to `discover_teams()` and `render_prompt()` in both `run()` and `run_scheduled()`. Switch template loading to use `TeamEntry.path` directly. Extract team names from `Vec<TeamEntry>` for the TUI call. Find the selected `TeamEntry` by name after TUI selection.

---

## Considerations

- `discover_teams()` currently returns `Vec<String>`. The TUI accepts `Vec<String>` for display. To avoid changing the TUI interface, extract names from `Vec<TeamEntry>` before passing to the TUI, and keep a reference to the full `Vec<TeamEntry>` for path lookup after selection.
- The user dir (`~/.claude-launch/user/`) must never be passed to `extract_dir()`. Double-check `resolve_workflow_dir()` to ensure no extract call touches paths under `user/`.
- `project_teams_dir` being `Some` but missing on disk is an error. `user_teams_dir` being missing on disk is not — user may not have defined any user-level teams yet. This asymmetry is intentional: the user explicitly configured `custom_dir`, so a missing directory is likely a misconfiguration.
- Both `run()` and `run_scheduled()` in `main.rs` build the template path and call `render_prompt()`. Both must be updated — it is easy to miss the scheduled path.
- The collision check must cover all three pairs: built-in vs user, built-in vs project, user vs project.
